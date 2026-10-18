use std::io::{self, Write};

use anyhow::bail;

use resource_loader::texture::{TextureFormat, TextureImage};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const HEADER_SIZE: usize = 80;
const LEVEL_INDEX_ENTRY_SIZE: usize = 24;
const SUPERCOMPRESSION_NONE: u32 = 0;
const WRITER_KEY: &str = "KTXwriter";
const WRITER: &str = concat!("launchpad ", env!("CARGO_PKG_VERSION"));
const WORD_BYTES: usize = 4;

/// Writes uncompressed textures and cube maps as KTX2 files, without supercompression.
pub struct Ktx2Writer;

impl Ktx2Writer {
    pub fn write(image: &TextureImage, output: &mut impl Write) -> anyhow::Result<()> {
        let Some(descriptor) = DataFormatDescriptor::new(image.format()) else {
            bail!("{} texels: only uncompressed formats can be written", image.format());
        };

        let descriptor = descriptor.bytes();
        let key_values = Ktx2Writer::key_values();
        let level_count = image.mip_levels() as usize;
        let descriptor_offset = HEADER_SIZE + level_count * LEVEL_INDEX_ENTRY_SIZE;
        let key_values_offset = descriptor_offset + descriptor.len();
        let alignment = Ktx2Writer::level_alignment(image.format());
        let level_sizes: Vec<usize> = (0..image.mip_levels())
            .map(|level| {
                image.layers().len()
                    * image
                        .format()
                        .level_size((image.width() >> level).max(1), (image.height() >> level).max(1))
            })
            .collect();

        // Levels are stored smallest first, each aligned for its texels.
        let mut level_offsets = vec![0; level_count];
        let mut position = key_values_offset + key_values.len();

        for level in (0..level_count).rev() {
            position = position.next_multiple_of(alignment);
            level_offsets[level] = position;
            position += level_sizes[level];
        }

        let mut bytes = KTX2_IDENTIFIER.to_vec();
        let face_count = image.layers().len() as u32;

        for word in [
            image.format().vulkan_format().value,
            DataFormatDescriptor::type_size(image.format()),
            image.width(),
            image.height(),
            0,
            0,
            face_count,
            image.mip_levels(),
            SUPERCOMPRESSION_NONE,
            descriptor_offset as u32,
            descriptor.len() as u32,
            key_values_offset as u32,
            key_values.len() as u32,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        bytes.resize(HEADER_SIZE, 0);

        for (offset, size) in level_offsets.iter().zip(&level_sizes) {
            for long in [*offset, *size, *size] {
                bytes.extend_from_slice(&(long as u64).to_le_bytes());
            }
        }

        bytes.extend_from_slice(&descriptor);
        bytes.extend_from_slice(&key_values);
        output.write_all(&bytes)?;
        let mut written = bytes.len();

        for level in (0..level_count).rev() {
            Ktx2Writer::pad(output, &mut written, level_offsets[level])?;
            let face_size = level_sizes[level] / image.layers().len();
            let start = (0..level)
                .map(|finer| level_sizes[finer] / image.layers().len())
                .sum::<usize>();

            for layer in image.layers() {
                output.write_all(&layer[start..start + face_size])?;
            }

            written += level_sizes[level];
        }

        Ok(())
    }

    fn key_values() -> Vec<u8> {
        let entry = format!("{WRITER_KEY}\0{WRITER}\0");
        let mut bytes = (entry.len() as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(entry.as_bytes());
        bytes.resize(bytes.len().next_multiple_of(WORD_BYTES), 0);

        bytes
    }

    // The specification aligns each level to the least common multiple of the texel size and 4.
    fn level_alignment(format: TextureFormat) -> usize {
        let texel = format.block_size();

        texel * WORD_BYTES / gcd(texel, WORD_BYTES)
    }

    fn pad(output: &mut impl Write, written: &mut usize, offset: usize) -> io::Result<()> {
        output.write_all(&vec![0; offset - *written])?;
        *written = offset;

        Ok(())
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

// The Khronos Data Format Descriptor of an uncompressed format: one basic block, one sample per channel.
struct DataFormatDescriptor {
    channels: &'static [u8],
    channel_bits: u32,
    numeric: Numeric,
    transfer: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Numeric {
    Unorm,
    Float,
}

impl DataFormatDescriptor {
    const BLOCK_HEADER_SIZE: usize = 24;
    const SAMPLE_SIZE: usize = 16;
    const VERSION: u16 = 2;
    const MODEL_RGBSDA: u8 = 1;
    const PRIMARIES_BT709: u8 = 1;
    const TRANSFER_LINEAR: u8 = 1;
    const TRANSFER_SRGB: u8 = 2;
    const RED: u8 = 0;
    const GREEN: u8 = 1;
    const BLUE: u8 = 2;
    const ALPHA: u8 = 15;
    const QUALIFIER_LINEAR: u8 = 0x10;
    const QUALIFIER_SIGNED: u8 = 0x40;
    const QUALIFIER_FLOAT: u8 = 0x80;
    // A float sample's bounds are the bit patterns of -1.0 and 1.0, whatever its size.
    const FLOAT_LOWER: u32 = 0xBF80_0000;
    const FLOAT_UPPER: u32 = 0x3F80_0000;
    const RGBA: &[u8] = &[Self::RED, Self::GREEN, Self::BLUE, Self::ALPHA];
    const RG: &[u8] = &[Self::RED, Self::GREEN];
    const R: &[u8] = &[Self::RED];

    fn new(format: TextureFormat) -> Option<DataFormatDescriptor> {
        let (channels, channel_bits, numeric, transfer) = match format {
            TextureFormat::Rg8Unorm => (Self::RG, 8, Numeric::Unorm, Self::TRANSFER_LINEAR),
            TextureFormat::Rgba8Unorm => (Self::RGBA, 8, Numeric::Unorm, Self::TRANSFER_LINEAR),
            TextureFormat::Rgba8Srgb => (Self::RGBA, 8, Numeric::Unorm, Self::TRANSFER_SRGB),
            TextureFormat::Rgba16Float => (Self::RGBA, 16, Numeric::Float, Self::TRANSFER_LINEAR),
            TextureFormat::R16Unorm => (Self::R, 16, Numeric::Unorm, Self::TRANSFER_LINEAR),
            TextureFormat::Rg16Unorm => (Self::RG, 16, Numeric::Unorm, Self::TRANSFER_LINEAR),
            TextureFormat::R32Float => (Self::R, 32, Numeric::Float, Self::TRANSFER_LINEAR),
            TextureFormat::Bc4Unorm | TextureFormat::Bc5Unorm | TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb => {
                return None;
            }
        };

        Some(DataFormatDescriptor {
            channels,
            channel_bits,
            numeric,
            transfer,
        })
    }

    // KTX2's typeSize: the size in bytes of the type each channel is stored as.
    fn type_size(format: TextureFormat) -> u32 {
        DataFormatDescriptor::new(format).map_or(1, |descriptor| descriptor.channel_bits / 8)
    }

    fn bytes(&self) -> Vec<u8> {
        let block_size = Self::BLOCK_HEADER_SIZE + Self::SAMPLE_SIZE * self.channels.len();
        let texel_bytes = self.channels.len() as u32 * self.channel_bits / 8;
        let mut bytes = ((WORD_BYTES + block_size) as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&Self::VERSION.to_le_bytes());
        bytes.extend_from_slice(&(block_size as u16).to_le_bytes());
        bytes.extend_from_slice(&[Self::MODEL_RGBSDA, Self::PRIMARIES_BT709, self.transfer, 0]);
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&[texel_bytes as u8, 0, 0, 0, 0, 0, 0, 0]);

        for (index, &channel) in self.channels.iter().enumerate() {
            let (qualifiers, lower, upper) = match self.numeric {
                Numeric::Unorm => (0, 0, u32::MAX >> (32 - self.channel_bits)),
                Numeric::Float => (
                    Self::QUALIFIER_FLOAT | Self::QUALIFIER_SIGNED,
                    Self::FLOAT_LOWER,
                    Self::FLOAT_UPPER,
                ),
            };
            let linear = if channel == Self::ALPHA && self.transfer == Self::TRANSFER_SRGB {
                Self::QUALIFIER_LINEAR
            } else {
                0
            };
            bytes.extend_from_slice(&((index as u32 * self.channel_bits) as u16).to_le_bytes());
            bytes.push((self.channel_bits - 1) as u8);
            bytes.push(channel | qualifiers | linear);
            bytes.extend_from_slice(&[0; 4]);
            bytes.extend_from_slice(&lower.to_le_bytes());
            bytes.extend_from_slice(&upper.to_le_bytes());
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use resource_loader::texture::{Ktx2Loader, TextureLoader};

    use super::*;

    const BC7_BLOCK_BYTES: usize = 16;

    // Every byte of a level's face holds the level and face it belongs to.
    fn tagged(level: u8, face: u8, size: usize) -> Vec<u8> {
        vec![level * 16 + face; size]
    }

    fn written(image: &TextureImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        Ktx2Writer::write(image, &mut bytes).unwrap();
        bytes
    }

    #[test]
    fn a_written_cube_map_reads_back_with_every_level() {
        let layers: Vec<Vec<u8>> = (0..6u8)
            .map(|face| {
                let mut layer = tagged(0, face, 4 * 4 * 2);
                layer.extend(tagged(1, face, 2 * 2 * 2));
                layer.extend(tagged(2, face, 2));
                layer
            })
            .collect();
        let image = TextureImage::new(TextureFormat::Rg8Unorm, 4, 4, 3, layers.clone()).unwrap();
        let read = Ktx2Loader::load(&written(&image)).unwrap();

        assert_eq!(read.format(), TextureFormat::Rg8Unorm);
        assert_eq!((read.width(), read.height(), read.mip_levels()), (4, 4, 3));
        assert_eq!(read.layers(), layers.as_slice());
    }

    #[test]
    fn every_uncompressed_format_reads_back_as_written() {
        for format in [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8Srgb,
            TextureFormat::Rgba16Float,
            TextureFormat::R16Unorm,
            TextureFormat::Rg16Unorm,
            TextureFormat::R32Float,
        ] {
            let size = format.level_size(2, 1) + format.level_size(1, 1);
            let layer: Vec<u8> = (0..size as u8).collect();
            let image = TextureImage::new(format, 2, 1, 2, vec![layer.clone()]).unwrap();
            let read = Ktx2Loader::load(&written(&image)).unwrap();

            assert_eq!(read.format(), format, "{format}");
            assert_eq!(read.layers(), &[layer], "{format}");
        }
    }

    #[test]
    fn a_written_file_describes_its_texels() {
        let image = TextureImage::new(TextureFormat::Rgba8Srgb, 1, 1, 1, vec![vec![0; 4]]).unwrap();
        let bytes = written(&image);
        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let descriptor = word(48) as usize;
        let block = descriptor + 4;
        let alpha_sample = block + 24 + 3 * 16;

        assert_eq!(word(16), 1, "typeSize");
        assert_eq!(word(descriptor) as usize, word(52) as usize);
        assert_eq!(&bytes[block + 8..block + 11], &[1, 1, 2], "RGBSDA, BT.709, sRGB");
        assert_eq!(bytes[block + 16], 4, "bytes per texel");
        assert_eq!(bytes[alpha_sample + 3], 0x1F, "linear alpha");
    }

    #[test]
    fn block_compressed_textures_are_not_written() {
        let image = TextureImage::new(TextureFormat::Bc7Srgb, 4, 4, 1, vec![vec![0; BC7_BLOCK_BYTES]]).unwrap();

        assert!(Ktx2Writer::write(&image, &mut Vec::new()).is_err());
    }
}
