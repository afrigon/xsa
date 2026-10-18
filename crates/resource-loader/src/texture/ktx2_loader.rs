use std::borrow::Cow;

use anyhow::{Context, bail, ensure};

use super::{TextureFormat, TextureImage, TextureLoader};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const VK_FORMAT_OFFSET: usize = 12;
const PIXEL_WIDTH_OFFSET: usize = 20;
const PIXEL_HEIGHT_OFFSET: usize = 24;
const PIXEL_DEPTH_OFFSET: usize = 28;
const LAYER_COUNT_OFFSET: usize = 32;
const FACE_COUNT_OFFSET: usize = 36;
const LEVEL_COUNT_OFFSET: usize = 40;
const SUPERCOMPRESSION_OFFSET: usize = 44;
const LEVEL_INDEX_OFFSET: usize = 80;
const LEVEL_INDEX_ENTRY_SIZE: usize = 24;
const LEVEL_BYTE_LENGTH_OFFSET: usize = 8;
const LEVEL_UNCOMPRESSED_LENGTH_OFFSET: usize = 16;
const WORD_BYTES: usize = 4;
const LONG_BYTES: usize = 8;
const SUPERCOMPRESSION_NONE: u32 = 0;
const SUPERCOMPRESSION_ZSTANDARD: u32 = 2;
const CUBE_FACE_COUNT: u32 = 6;

/// Reads KTX2 files holding a 2D texture or a cube map, uncompressed or with Zstandard supercompression.
pub struct Ktx2Loader;

impl TextureLoader for Ktx2Loader {
    const EXTENSION: &'static str = "ktx2";

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage> {
        ensure!(
            bytes.len() >= LEVEL_INDEX_OFFSET && &bytes[..KTX2_IDENTIFIER.len()] == KTX2_IDENTIFIER,
            "not a KTX2 file"
        );

        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + WORD_BYTES].try_into().unwrap());
        let vulkan_format = word(VK_FORMAT_OFFSET);
        let format = TextureFormat::from_vulkan_format(vulkan_format)
            .with_context(|| format!("unsupported Vulkan format {vulkan_format}"))?;
        ensure!(word(PIXEL_DEPTH_OFFSET) == 0, "3D textures are not supported");
        ensure!(word(LAYER_COUNT_OFFSET) == 0, "texture arrays are not supported");

        let face_count = word(FACE_COUNT_OFFSET);
        ensure!(
            face_count == 1 || face_count == CUBE_FACE_COUNT,
            "{face_count} faces: expected 1 or {CUBE_FACE_COUNT}"
        );

        // A level count of 0 asks the reader to generate the mip chain below the one level stored.
        let mip_levels = word(LEVEL_COUNT_OFFSET).max(1);
        let supercompression = word(SUPERCOMPRESSION_OFFSET);
        let mut image = TextureImage {
            format,
            width: word(PIXEL_WIDTH_OFFSET),
            height: word(PIXEL_HEIGHT_OFFSET),
            mip_levels,
            layers: Vec::new(),
        };
        let mut layers = vec![Vec::with_capacity(image.layer_size()); face_count as usize];

        for level in 0..mip_levels {
            let data = Ktx2Loader::level(bytes, level, supercompression)?;
            let face_size = image.format.level_size(image.width >> level, image.height >> level);
            ensure!(
                data.len() == face_size * face_count as usize,
                "level {level} holds {} bytes, expected {}",
                data.len(),
                face_size * face_count as usize
            );

            for (layer, face) in layers.iter_mut().zip(data.chunks_exact(face_size)) {
                layer.extend_from_slice(face);
            }
        }

        image.layers = layers;
        Ok(image)
    }
}

impl Ktx2Loader {
    // A level's data, all faces, decompressed.
    fn level(bytes: &[u8], level: u32, supercompression: u32) -> anyhow::Result<Cow<'_, [u8]>> {
        let entry = LEVEL_INDEX_OFFSET + level as usize * LEVEL_INDEX_ENTRY_SIZE;
        ensure!(
            bytes.len() >= entry + LEVEL_INDEX_ENTRY_SIZE,
            "the level index is truncated"
        );

        let long = |offset: usize| {
            u64::from_le_bytes(bytes[entry + offset..entry + offset + LONG_BYTES].try_into().unwrap()) as usize
        };
        let start = long(0);
        let length = long(LEVEL_BYTE_LENGTH_OFFSET);
        let data = bytes
            .get(start..start + length)
            .with_context(|| format!("level {level} is truncated"))?;

        match supercompression {
            SUPERCOMPRESSION_NONE => Ok(Cow::Borrowed(data)),
            SUPERCOMPRESSION_ZSTANDARD => Ok(Cow::Owned(
                zstd::bulk::decompress(data, long(LEVEL_UNCOMPRESSED_LENGTH_OFFSET))
                    .with_context(|| format!("decompressing level {level}"))?,
            )),
            other => bail!("unsupported supercompression scheme {other}"),
        }
    }
}
