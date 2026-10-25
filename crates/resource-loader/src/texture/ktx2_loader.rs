use std::collections::BTreeMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use anyhow::{Context, bail, ensure};

use super::{TextureFormat, TextureImage, TextureKind, TextureLoader, TextureMetadata, VulkanFormat};

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
/// supercompression, with their key/value metadata. Each level is stored apart, so a range of levels reads without the
/// others.
pub struct Ktx2Loader;

impl TextureLoader for Ktx2Loader {
    const EXTENSION: &'static str = "ktx2";

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage> {
        Ktx2Loader::read(Cursor::new(bytes), 0)
    }
}

impl Ktx2Loader {
    /// Reads the levels from `first_level` down to the smallest, seeking past the finer levels' data.
    pub fn read(mut reader: impl Read + Seek, first_level: u32) -> anyhow::Result<TextureImage> {
        let mut header = [0; LEVEL_INDEX_OFFSET];
        reader.read_exact(&mut header).context("not a KTX2 file")?;
        ensure!(&header[..KTX2_IDENTIFIER.len()] == KTX2_IDENTIFIER, "not a KTX2 file");

        let word = |offset: usize| u32::from_le_bytes(header[offset..offset + WORD_BYTES].try_into().unwrap());
        let vulkan_format = VulkanFormat {
            value: word(VK_FORMAT_OFFSET),
        };
        let format = TextureFormat::from_vulkan_format(vulkan_format)
            .with_context(|| format!("unsupported Vulkan format {}", vulkan_format.value))?;
        ensure!(word(PIXEL_DEPTH_OFFSET) == 0, "3D textures are not supported");

        let face_count = word(FACE_COUNT_OFFSET);
        let array_layer_count = word(LAYER_COUNT_OFFSET);
        let kind = match (array_layer_count, face_count) {
            (0, 1) => TextureKind::Plane,
            (0, CUBE_FACE_COUNT) => TextureKind::CubeMap,
            (_, 1) => TextureKind::Array,
            (_, CUBE_FACE_COUNT) => bail!("cube map arrays are not supported"),
            _ => bail!("{face_count} faces: expected 1 or {CUBE_FACE_COUNT}"),
        };
        // Each level holds every array layer, and within each layer every face.
        let layer_count = array_layer_count.max(1) * face_count;

        // A level count of 0 asks the reader to generate the mip chain below the one level stored.
        let mip_levels = word(LEVEL_COUNT_OFFSET).max(1);
        let supercompression = word(SUPERCOMPRESSION_OFFSET);
        let level_index = Ktx2Loader::level_index(&mut reader, mip_levels)?;
        let metadata = Ktx2Loader::metadata(
            &mut reader,
            word(KEY_VALUE_OFFSET_OFFSET),
            word(KEY_VALUE_LENGTH_OFFSET),
        )?;
        let mut image = TextureImage {
            format,
            kind,
            width: word(PIXEL_WIDTH_OFFSET),
            height: word(PIXEL_HEIGHT_OFFSET),
            mip_levels,
            first_level,
            layers: Vec::new(),
            metadata: TextureMetadata::new(metadata),
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

        for level in first_level..mip_levels {
            let face_size = image.format.level_size(image.width >> level, image.height >> level);
            let level_size = face_size * layer_count as usize;
            let data = Ktx2Loader::level(
                &mut reader,
                &level_index[level as usize],
                level,
                level_size,
                supercompression,
            )?;
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

    fn level_index(reader: &mut impl Read, mip_levels: u32) -> anyhow::Result<Vec<[u8; LEVEL_INDEX_ENTRY_SIZE]>> {
        (0..mip_levels)
            .map(|_| {
                let mut entry = [0; LEVEL_INDEX_ENTRY_SIZE];
                reader.read_exact(&mut entry).context("the level index is truncated")?;
                Ok(entry)
            })
            .collect()
    }

    // Each entry is its length, then its key ending with a NUL, then its value, padded to a multiple of 4 bytes.
    fn metadata(
        reader: &mut (impl Read + Seek),
        offset: u32,
        length: u32,
    ) -> anyhow::Result<BTreeMap<String, Vec<u8>>> {
        let mut bytes = Vec::new();

        if length > 0 {
            reader.seek(SeekFrom::Start(u64::from(offset)))?;
            reader
                .take(u64::from(length))
                .read_to_end(&mut bytes)
                .context("reading the key/value data")?;
            ensure!(bytes.len() == length as usize, "the key/value data is truncated");
        }

        let mut data = bytes.as_slice();
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
    fn level(
        reader: &mut (impl Read + Seek),
        entry: &[u8; LEVEL_INDEX_ENTRY_SIZE],
        level: u32,
        size: usize,
        supercompression: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let long = |offset: usize| u64::from_le_bytes(entry[offset..offset + LONG_BYTES].try_into().unwrap());
        let length = long(LEVEL_BYTE_LENGTH_OFFSET);
        reader
            .seek(SeekFrom::Start(long(0)))
            .with_context(|| format!("seeking to level {level}"))?;
        let mut data = reader.take(length);
        let mut level_data = Vec::new();
        level_data
            .try_reserve_exact(size)
            .with_context(|| format!("level {level}'s {size} bytes cannot be allocated"))?;

        match supercompression {
            SUPERCOMPRESSION_NONE => {
                data.read_to_end(&mut level_data)
                    .with_context(|| format!("reading level {level}"))?;
                ensure!(level_data.len() as u64 == length, "level {level} is truncated");
            }
            SUPERCOMPRESSION_ZSTANDARD => {
                let uncompressed_length = long(LEVEL_UNCOMPRESSED_LENGTH_OFFSET) as usize;
                ensure!(
                    uncompressed_length == size,
                    "level {level} decompresses to {uncompressed_length} bytes, expected {size}"
                );

                zstd::stream::read::Decoder::new(data)
                    .and_then(|decoder| decoder.take(size as u64).read_to_end(&mut level_data))
                    .with_context(|| format!("decompressing level {level}"))?;
            }
            other => bail!("unsupported supercompression scheme {other}"),
        }

        Ok(level_data)
    }
}
