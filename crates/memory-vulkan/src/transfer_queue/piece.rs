use std::collections::VecDeque;

use ash::vk;

use crate::transfer_queue::{CopyRegion, PieceCopy, STAGING_BLOCK_SIZE};
use crate::{ImageUpload, MemoryError};

// A queue without graphics or compute copies a buffer to an image only from offsets that are multiples of 4.
const IMAGE_COPY_ALIGNMENT: usize = 4;
const BYTE_ALIGNMENT: usize = 1;

// The unit requests are interleaved and budgeted in: copies whose staged bytes, padding included, fit one staging
// block.
#[derive(Debug, Default)]
pub(crate) struct Piece {
    pub copies: Vec<PieceCopy>,
    pub size: usize,
}

impl Piece {
    // Splits each copy into bands of whole rows of texel blocks.
    pub fn split_image(upload: &ImageUpload) -> Result<VecDeque<Piece>, MemoryError> {
        let block = upload.texel_block;
        let alignment = Piece::least_common_multiple(block.size as usize, IMAGE_COPY_ALIGNMENT);
        let mut copies = Vec::new();

        for copy in &upload.copies {
            let row_size = block.row_size(copy.extent.width);
            let rows = block.rows(copy.extent.height);
            let data = upload
                .data
                .get(copy.data)
                .ok_or_else(|| Piece::invalid("a copy names a byte array the upload does not have"))?;

            if copy.offset + rows as usize * row_size > (**data).as_ref().len() {
                return Err(Piece::invalid("a copy reads past the end of its bytes"));
            }

            let rows_per_band = ((STAGING_BLOCK_SIZE - (alignment - 1)) / row_size) as u32;

            if rows_per_band == 0 {
                return Err(Piece::invalid("a row of texel blocks is larger than a staging block"));
            }

            let mut row = 0;

            while row < rows {
                let band = (rows - row).min(rows_per_band);
                let top = row * block.extent.height;
                copies.push(PieceCopy {
                    data: copy.data,
                    data_offset: copy.offset + row as usize * row_size,
                    size: band as usize * row_size,
                    alignment,
                    region: CopyRegion::Image {
                        mip_level: copy.mip_level,
                        array_layer: copy.array_layer,
                        offset: vk::Offset3D {
                            y: copy.image_offset.y + top as i32,
                            ..copy.image_offset
                        },
                        extent: vk::Extent2D {
                            width: copy.extent.width,
                            height: (band * block.extent.height).min(copy.extent.height - top),
                        },
                    },
                });
                row += band;
            }
        }

        Ok(Piece::pack(copies))
    }

    // `size` bytes at the same offsets in the request's only byte array and in the buffer.
    pub fn split_bytes(size: usize) -> VecDeque<Piece> {
        let copies = (0..size).step_by(STAGING_BLOCK_SIZE).map(|start| PieceCopy {
            data: 0,
            data_offset: start,
            size: (size - start).min(STAGING_BLOCK_SIZE),
            alignment: BYTE_ALIGNMENT,
            region: CopyRegion::Buffer { offset: start as u64 },
        });

        Piece::pack(copies)
    }

    fn pack(copies: impl IntoIterator<Item = PieceCopy>) -> VecDeque<Piece> {
        let mut pieces = VecDeque::new();
        let mut current = Piece::default();

        for copy in copies {
            let staged = copy.size + copy.alignment - 1;

            if !current.copies.is_empty() && current.size + staged > STAGING_BLOCK_SIZE {
                pieces.push_back(std::mem::take(&mut current));
            }

            current.size += staged;
            current.copies.push(copy);
        }

        if !current.copies.is_empty() {
            pieces.push_back(current);
        }

        pieces
    }

    fn least_common_multiple(a: usize, b: usize) -> usize {
        let (mut x, mut y) = (a, b);

        while y != 0 {
            (x, y) = (y, x % y);
        }

        a / x * b
    }

    fn invalid(reason: &'static str) -> MemoryError {
        MemoryError::HostAccess {
            buffer: "upload data".to_owned(),
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageCopy, TexelBlock};

    const BC7: TexelBlock = TexelBlock {
        extent: vk::Extent2D { width: 4, height: 4 },
        size: 16,
    };

    fn level_size(width: u32, height: u32) -> usize {
        BC7.rows(height) as usize * BC7.row_size(width)
    }

    fn upload(width: u32, height: u32) -> ImageUpload {
        upload_of(width, height, level_size(width, height))
    }

    fn upload_of(width: u32, height: u32, size: usize) -> ImageUpload {
        ImageUpload {
            data: vec![Box::new(vec![0; size])],
            copies: vec![ImageCopy {
                data: 0,
                offset: 0,
                mip_level: 0,
                array_layer: 0,
                image_offset: vk::Offset3D::default(),
                extent: vk::Extent2D { width, height },
            }],
            texel_block: BC7,
        }
    }

    fn bands(pieces: &VecDeque<Piece>) -> Vec<(i32, u32)> {
        pieces
            .iter()
            .flat_map(|piece| &piece.copies)
            .map(|copy| match copy.region {
                CopyRegion::Image { offset, extent, .. } => (offset.y, extent.height),
                CopyRegion::Buffer { .. } => unreachable!("an image upload copies to an image"),
            })
            .collect()
    }

    #[test]
    fn a_level_larger_than_a_block_splits_into_bands_of_whole_block_rows() {
        let pieces = Piece::split_image(&upload(8192, 8192)).unwrap();

        assert!(pieces.iter().all(|piece| piece.size <= STAGING_BLOCK_SIZE));
        assert_eq!(
            bands(&pieces),
            vec![(0, 2044), (2044, 2044), (4088, 2044), (6132, 2044), (8176, 16)]
        );
    }

    #[test]
    fn small_levels_share_a_piece_and_keep_their_texel_extent() {
        let mut upload = upload_of(2, 2, 32);
        upload.copies.push(ImageCopy {
            offset: 16,
            mip_level: 1,
            extent: vk::Extent2D { width: 1, height: 1 },
            ..upload.copies[0]
        });
        let pieces = Piece::split_image(&upload).unwrap();

        assert_eq!(pieces.len(), 1);
        assert_eq!(bands(&pieces), vec![(0, 2), (0, 1)]);
    }

    #[test]
    fn a_copy_past_its_bytes_is_refused() {
        let upload = upload_of(16, 16, level_size(16, 16) - 1);

        assert!(Piece::split_image(&upload).is_err());
    }

    #[test]
    fn bytes_split_at_block_boundaries() {
        let pieces = Piece::split_bytes(STAGING_BLOCK_SIZE + 1);

        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[1].copies[0].data_offset, STAGING_BLOCK_SIZE);
        assert_eq!(pieces[1].copies[0].size, 1);
    }
}
