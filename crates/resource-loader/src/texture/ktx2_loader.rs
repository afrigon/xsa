use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Read;

use anyhow::{Context, bail, ensure};

use super::{TextureFormat, TextureImage, TextureLoader, TextureMetadata, TextureShape, VulkanFormat};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const VK_FORMAT_OFFSET: usize = 12;
const PIXEL_WIDTH_OFFSET: usize = 20;
const PIXEL_HEIGHT_OFFSET: usize = 24;
const PIXEL_DEPTH_OFFSET: usize = 28;
const LAYER_COUNT_OFFSET: usize = 32;
const FACE_COUNT_OFFSET: usize = 36;
const LEVEL_COUNT_OFFSET: usize = 40;
const SUPERCOMPRESSION_OFFSET: usize = 44;
const KEY_VALUE_OFFSET_OFFSET: usize = 56;
const KEY_VALUE_LENGTH_OFFSET: usize = 60;
const LEVEL_INDEX_OFFSET: usize = 80;
const LEVEL_INDEX_ENTRY_SIZE: usize = 24;
const LEVEL_BYTE_LENGTH_OFFSET: usize = 8;
const LEVEL_UNCOMPRESSED_LENGTH_OFFSET: usize = 16;
const WORD_BYTES: usize = 4;
const LONG_BYTES: usize = 8;
const SUPERCOMPRESSION_NONE: u32 = 0;
const SUPERCOMPRESSION_ZSTANDARD: u32 = 2;
const CUBE_FACE_COUNT: u32 = 6;

/// Reads KTX2 files holding a 2D texture, a cube map or a texture array, uncompressed or with Zstandard
/// supercompression, with their key/value metadata.
pub struct Ktx2Loader;

impl TextureLoader for Ktx2Loader {
    const EXTENSION: &'static str = "ktx2";

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage> {
        ensure!(
            bytes.len() >= LEVEL_INDEX_OFFSET && &bytes[..KTX2_IDENTIFIER.len()] == KTX2_IDENTIFIER,
            "not a KTX2 file"
        );

        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + WORD_BYTES].try_into().unwrap());
        let vulkan_format = VulkanFormat {
            value: word(VK_FORMAT_OFFSET),
        };
        let format = TextureFormat::from_vulkan_format(vulkan_format)
            .with_context(|| format!("unsupported Vulkan format {}", vulkan_format.value))?;
        ensure!(word(PIXEL_DEPTH_OFFSET) == 0, "3D textures are not supported");

        let face_count = word(FACE_COUNT_OFFSET);
        let array_layer_count = word(LAYER_COUNT_OFFSET);
        let shape = match (array_layer_count, face_count) {
            (0, 1) => TextureShape::Plane,
            (0, CUBE_FACE_COUNT) => TextureShape::CubeMap,
            (_, 1) => TextureShape::Array,
            (_, CUBE_FACE_COUNT) => bail!("cube map arrays are not supported"),
            _ => bail!("{face_count} faces: expected 1 or {CUBE_FACE_COUNT}"),
        };
        // Each level holds every array layer, and within each layer every face.
        let layer_count = array_layer_count.max(1) * face_count;

        // A level count of 0 asks the reader to generate the mip chain below the one level stored.
        let mip_levels = word(LEVEL_COUNT_OFFSET).max(1);
        let supercompression = word(SUPERCOMPRESSION_OFFSET);
        let mut image = TextureImage {
            format,
            shape,
            width: word(PIXEL_WIDTH_OFFSET),
            height: word(PIXEL_HEIGHT_OFFSET),
            mip_levels,
            layers: Vec::new(),
            metadata: TextureMetadata::new(Ktx2Loader::metadata(
                bytes,
                word(KEY_VALUE_OFFSET_OFFSET),
                word(KEY_VALUE_LENGTH_OFFSET),
            )?),
        };
        let layer_size = image.layer_size()?;
        let mut layers = Vec::new();

        for _ in 0..layer_count {
            let mut layer = Vec::new();
            layer
                .try_reserve_exact(layer_size)
                .with_context(|| format!("{layer_size} bytes per face cannot be allocated"))?;
            layers.push(layer);
        }

        for level in 0..mip_levels {
            let face_size = image.format.level_size(image.width >> level, image.height >> level);
            let level_size = face_size * layer_count as usize;
            let data = Ktx2Loader::level(bytes, level, level_size, supercompression)?;
            ensure!(
                data.len() == level_size,
                "level {level} holds {} bytes, expected {level_size}",
                data.len()
            );

            for (layer, face) in layers.iter_mut().zip(data.chunks_exact(face_size)) {
                layer.extend_from_slice(face);
            }
        }

        image.layers = layers;
        image.check_layer_count()?;
        Ok(image)
    }
}

impl Ktx2Loader {
    // Each entry is its length, then its key ending with a NUL, then its value, padded to a multiple of 4 bytes.
    fn metadata(bytes: &[u8], offset: u32, length: u32) -> anyhow::Result<BTreeMap<String, Vec<u8>>> {
        let start = offset as usize;
        let mut data = start
            .checked_add(length as usize)
            .and_then(|end| bytes.get(start..end))
            .context("the key/value data is truncated")?;
        let mut metadata = BTreeMap::new();

        while !data.is_empty() {
            let (entry_length, rest) = data
                .split_first_chunk::<WORD_BYTES>()
                .context("a key/value entry's length is truncated")?;
            let entry_length = u32::from_le_bytes(*entry_length) as usize;
            let entry = rest.get(..entry_length).context("a key/value entry is truncated")?;
            let key_end = entry
                .iter()
                .position(|&byte| byte == 0)
                .context("a key/value entry's key has no NUL")?;
            let key = std::str::from_utf8(&entry[..key_end]).context("a key/value entry's key is not UTF-8")?;
            metadata.insert(key.to_string(), entry[key_end + 1..].to_vec());

            let padded_length = entry_length.next_multiple_of(WORD_BYTES);
            data = rest.get(padded_length..).unwrap_or_default();
        }

        Ok(metadata)
    }

    // A level's data, all faces, decompressed; `size` is what the header's shape says it holds.
    fn level(bytes: &[u8], level: u32, size: usize, supercompression: u32) -> anyhow::Result<Cow<'_, [u8]>> {
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
        let data = start
            .checked_add(length)
            .and_then(|end| bytes.get(start..end))
            .with_context(|| format!("level {level} is truncated"))?;

        match supercompression {
            SUPERCOMPRESSION_NONE => Ok(Cow::Borrowed(data)),
            SUPERCOMPRESSION_ZSTANDARD => {
                let uncompressed_length = long(LEVEL_UNCOMPRESSED_LENGTH_OFFSET);
                ensure!(
                    uncompressed_length == size,
                    "level {level} decompresses to {uncompressed_length} bytes, expected {size}"
                );

                let mut level_data = Vec::new();
                level_data
                    .try_reserve_exact(size)
                    .with_context(|| format!("level {level}'s {size} bytes cannot be allocated"))?;
                zstd::stream::read::Decoder::new(data)
                    .and_then(|decoder| decoder.take(size as u64).read_to_end(&mut level_data))
                    .with_context(|| format!("decompressing level {level}"))?;

                Ok(Cow::Owned(level_data))
            }
            other => bail!("unsupported supercompression scheme {other}"),
        }
    }
}
