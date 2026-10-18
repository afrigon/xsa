use std::io::Write;
use std::num::NonZero;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use anyhow::Context;
use resource_loader::terrain::{CubeFace, LevelRecord, PageAddress, PageEntry, PageKey, TerrainHeader};

use super::{CubeMapInput, EncodedPage, LayerEncoding, PageBuilder};

/// Writes a cube map as a `.terrain` file: every page of every level, encoded on every core.
pub struct TerrainWriter<'a> {
    input: &'a CubeMapInput,
    encoding: LayerEncoding,
}

impl<'a> TerrainWriter<'a> {
    // Decompression is as fast at any level, and pages are written once and read many times.
    const COMPRESSION_LEVEL: i32 = 19;

    pub fn new(input: &'a CubeMapInput, encoding: LayerEncoding) -> TerrainWriter<'a> {
        TerrainWriter { input, encoding }
    }

    pub fn write(&self, output: &mut impl Write) -> anyhow::Result<()> {
        let addresses = self.page_addresses();
        let pages = self.encoded_pages(&addresses)?;
        let header = TerrainHeader {
            format: self.encoding.page_format(),
            encoding: self.encoding.page_encoding(),
            level_count: self.input.level_count(),
        };
        let entry_size = PageEntry::size(header.encoding) as u64;
        let record_count = (CubeFace::ALL.len() * header.level_count as usize) as u64;
        let entries_start = TerrainHeader::SIZE as u64 + record_count * LevelRecord::SIZE as u64;
        let mut entries_offset = entries_start;
        let mut data_offset = entries_start + pages.len() as u64 * entry_size;
        header.write(output)?;

        for _ in CubeFace::ALL {
            for level in 0..header.level_count {
                let entry_count = TerrainWriter::level_page_count(level) as u64;
                LevelRecord {
                    entries_offset,
                    entry_count,
                }
                .write(output)?;
                entries_offset += entry_count * entry_size;
            }
        }

        for (address, page) in addresses.iter().zip(&pages) {
            PageEntry {
                key: address.key,
                data_offset,
                data_size: page.data.len() as u32,
                range: page.range,
            }
            .write(output)?;
            data_offset += page.data.len() as u64;
        }

        for page in &pages {
            output.write_all(&page.data)?;
        }

        Ok(())
    }

    /// The number of pages the file holds.
    pub fn page_count(&self) -> usize {
        CubeFace::ALL.len()
            * (0..self.input.level_count())
                .map(TerrainWriter::level_page_count)
                .sum::<usize>()
    }

    fn level_page_count(level: u32) -> usize {
        1 << (2 * level)
    }

    // Every page in file order: by face, then level, then key. A level's pages cover its whole face, so its keys are
    // every Morton code below its page count.
    fn page_addresses(&self) -> Vec<PageAddress> {
        CubeFace::ALL
            .into_iter()
            .flat_map(|face| {
                (0..self.input.level_count()).flat_map(move |level| {
                    (0..TerrainWriter::level_page_count(level) as u64).map(move |code| PageAddress {
                        face,
                        level,
                        key: PageKey { code },
                    })
                })
            })
            .collect()
    }

    // Each page compressed, in the order of the addresses.
    fn encoded_pages(&self, addresses: &[PageAddress]) -> anyhow::Result<Vec<EncodedPage>> {
        let builders: Vec<PageBuilder> = (0..self.input.level_count())
            .map(|level| PageBuilder::new(self.input, level, self.encoding))
            .collect();
        let pages: Vec<OnceLock<EncodedPage>> = addresses.iter().map(|_| OnceLock::new()).collect();
        let next = AtomicUsize::new(0);
        let workers = thread::available_parallelism().map_or(1, NonZero::get);

        thread::scope(|scope| {
            let encoders: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| -> anyhow::Result<()> {
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(address) = addresses.get(index) else {
                                return Ok(());
                            };
                            let texels = builders[address.level as usize].page(address.face, address.key);
                            let page = self.encoding.encode(&texels);
                            let data = zstd::bulk::compress(&page.data, Self::COMPRESSION_LEVEL)
                                .with_context(|| format!("compressing page {address:?}"))?;
                            let _ = pages[index].set(EncodedPage { data, ..page });
                        }
                    })
                })
                .collect();

            encoders
                .into_iter()
                .try_for_each(|encoder| encoder.join().expect("a page encoder panicked"))
        })?;

        Ok(pages.into_iter().map(|page| page.into_inner().unwrap()).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use glam::DVec3;
    use resource_loader::terrain::{PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, PageEncoding, TerrainReader};
    use resource_loader::texture::TextureFormat;

    use super::*;
    use crate::terrain::TestCubeMap;

    const FACE_EXTENT: u32 = 256;
    const HEIGHT_GRADIENT: DVec3 = DVec3::new(1000.0, -2000.0, 3000.0);

    struct LayerCase {
        input: TextureFormat,
        pages: TextureFormat,
        texel: Vec<u8>,
    }

    // The direction from the body's centre through a texel's centre, `x` and `y` possibly beyond the face.
    fn direction(face: CubeFace, extent: u32, x: i32, y: i32) -> DVec3 {
        let along = |coordinate: i32| f64::from(2 * coordinate + 1) / f64::from(extent) - 1.0;
        let point =
            face.normal().as_dvec3() + face.column_axis().as_dvec3() * along(x) + face.row_axis().as_dvec3() * along(y);

        point.normalize()
    }

    fn height(direction: DVec3) -> f32 {
        direction.dot(HEIGHT_GRADIENT) as f32
    }

    fn converted(input: &CubeMapInput, encoding: PageEncoding) -> TerrainReader<Cursor<Vec<u8>>> {
        let encoding = LayerEncoding::new(encoding, input.format()).unwrap();
        let mut file = Vec::new();
        TerrainWriter::new(input, encoding).write(&mut file).unwrap();

        TerrainReader::new(Cursor::new(file)).unwrap()
    }

    #[test]
    fn heights_read_back_with_their_borders_and_ranges() {
        let input = TestCubeMap::full_chain(TextureFormat::R32Float, FACE_EXTENT)
            .input(|face, extent, x, y| {
                height(direction(face, extent, x as i32, y as i32))
                    .to_le_bytes()
                    .to_vec()
            })
            .unwrap();
        let mut reader = converted(&input, PageEncoding::PageHeights);
        let header = *reader.header();

        assert_eq!(header.format, TextureFormat::R16Unorm);
        assert_eq!(header.encoding, PageEncoding::PageHeights);
        assert_eq!(header.level_count, 2);

        for face in CubeFace::ALL {
            for level in 0..header.level_count {
                let extent = PAGE_INTERIOR << level;
                let entries = reader.entries(face, level).unwrap();
                assert_eq!(entries.len(), 1 << (2 * level));
                assert!(entries.is_sorted_by_key(|entry| entry.key));

                for entry in entries {
                    let range = entry.range.unwrap();
                    let page = reader.page(&entry).unwrap();
                    assert_eq!(page.len(), (PAGE_EXTENT * PAGE_EXTENT) as usize * size_of::<u16>());

                    let values: Vec<u16> = page
                        .as_chunks()
                        .0
                        .iter()
                        .map(|value| u16::from_le_bytes(*value))
                        .collect();
                    assert_eq!(values.iter().min(), Some(&0));
                    assert_eq!(values.iter().max(), Some(&u16::MAX));

                    let step = (range.maximum - range.minimum) / f32::from(u16::MAX);
                    // Beyond a face edge, a border texel comes from a texel a few texels away on the cube; a rotated
                    // or mirrored edge would be off by thousands of metres.
                    let edge_tolerance = (HEIGHT_GRADIENT.length() * 16.0 / f64::from(extent)) as f32;
                    let inside = |coordinate: i32| (0..extent as i32).contains(&coordinate);

                    for (index, value) in values.iter().enumerate() {
                        let column = (index as u32 % PAGE_EXTENT) as i32 - PAGE_BORDER as i32;
                        let row = (index as u32 / PAGE_EXTENT) as i32 - PAGE_BORDER as i32;
                        let x = (entry.key.x() * PAGE_INTERIOR) as i32 + column;
                        let y = (entry.key.y() * PAGE_INTERIOR) as i32 + row;
                        let expected = height(direction(face, extent, x, y));
                        let tolerance = if inside(x) && inside(y) { step } else { edge_tolerance };

                        assert!(
                            (range.height(*value) - expected).abs() <= tolerance,
                            "{face:?} level {level} page {:?} texel ({column}, {row}): {} instead of {expected}",
                            entry.key,
                            range.height(*value)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn colours_normals_and_hapke_maps_keep_their_texel_formats() {
        let cases = [
            LayerCase {
                input: TextureFormat::Rgba8Srgb,
                pages: TextureFormat::Bc7Srgb,
                texel: vec![200, 100, 50, 255],
            },
            LayerCase {
                input: TextureFormat::Rg8Unorm,
                pages: TextureFormat::Bc5Unorm,
                texel: vec![128, 64],
            },
            LayerCase {
                input: TextureFormat::Rgba16Float,
                pages: TextureFormat::Rgba16Float,
                texel: vec![0, 60, 0, 56, 0, 52, 0, 60],
            },
        ];

        for case in cases {
            let input = TestCubeMap::full_chain(case.input, FACE_EXTENT)
                .input(|_, _, _, _| case.texel.clone())
                .unwrap();
            let mut reader = converted(&input, PageEncoding::Direct);
            assert_eq!(reader.header().format, case.pages);
            assert_eq!(reader.header().encoding, PageEncoding::Direct);

            let entry = reader.entries(CubeFace::NegativeZ, 1).unwrap()[3];
            assert!(entry.range.is_none());
            assert_eq!(
                reader.page(&entry).unwrap().len(),
                case.pages.level_size(PAGE_EXTENT, PAGE_EXTENT)
            );
        }
    }
}
