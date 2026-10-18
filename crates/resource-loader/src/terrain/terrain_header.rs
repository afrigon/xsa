use anyhow::{Context, ensure};

use super::{PAGE_BORDER, PAGE_EXTENT, PageEncoding};
use crate::texture::{TextureFormat, VulkanFormat};

/// The start of a `.terrain` file: what every page holds and how many levels there are.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TerrainHeader {
    pub format: TextureFormat,
    pub encoding: PageEncoding,
    pub level_count: u32,
}

impl TerrainHeader {
    /// The size of the header in bytes.
    pub const SIZE: usize = 28;
    /// The most levels a file holds: its finest face, `PAGE_INTERIOR << 24` texels, is the widest a `u32` holds.
    pub const MAXIMUM_LEVEL_COUNT: u32 = 25;
    /// The bytes a `.terrain` file starts with.
    pub const MAGIC: &[u8; 4] = b"XSAT";
    /// The version of the format this crate reads.
    pub const VERSION: u32 = 1;
    const VERSION_OFFSET: usize = 4;
    const FORMAT_OFFSET: usize = 8;
    const ENCODING_OFFSET: usize = 12;
    const PAGE_EXTENT_OFFSET: usize = 16;
    const BORDER_OFFSET: usize = 20;
    const LEVEL_COUNT_OFFSET: usize = 24;

    pub fn read(bytes: &[u8; TerrainHeader::SIZE]) -> anyhow::Result<TerrainHeader> {
        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + size_of::<u32>()].try_into().unwrap());
        ensure!(&bytes[..Self::MAGIC.len()] == Self::MAGIC, "not a terrain file");

        let version = word(Self::VERSION_OFFSET);
        ensure!(
            version == Self::VERSION,
            "terrain file version {version}: expected {}",
            Self::VERSION
        );

        let page_extent = word(Self::PAGE_EXTENT_OFFSET);
        let border = word(Self::BORDER_OFFSET);
        ensure!(
            page_extent == PAGE_EXTENT && border == PAGE_BORDER,
            "{page_extent}-texel pages with a {border}-texel border: expected {PAGE_EXTENT} and {PAGE_BORDER}"
        );

        let level_count = word(Self::LEVEL_COUNT_OFFSET);
        ensure!(
            level_count <= Self::MAXIMUM_LEVEL_COUNT,
            "{level_count} levels: a terrain file holds at most {}",
            Self::MAXIMUM_LEVEL_COUNT
        );

        let vulkan_format = VulkanFormat {
            value: word(Self::FORMAT_OFFSET),
        };

        Ok(TerrainHeader {
            format: TextureFormat::from_vulkan_format(vulkan_format)
                .with_context(|| format!("unsupported Vulkan format {}", vulkan_format.value))?,
            encoding: PageEncoding::from_value(word(Self::ENCODING_OFFSET))?,
            level_count,
        })
    }

    /// The size in bytes of a page's data once decompressed.
    pub fn page_size(&self) -> usize {
        self.format.level_size(PAGE_EXTENT, PAGE_EXTENT)
    }
}
