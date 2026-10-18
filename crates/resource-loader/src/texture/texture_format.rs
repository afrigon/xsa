use std::fmt::{self, Display, Formatter};

use anyhow::bail;

use super::ColorSpace;

const COMPRESSED_BLOCK_EXTENT: u32 = 4;
const BC4_BLOCK_BYTES: usize = 8;
const BC5_BC7_BLOCK_BYTES: usize = 16;
const RGBA8_TEXEL_BYTES: usize = 4;
const RGBA16_FLOAT_TEXEL_BYTES: usize = 8;
const R16_TEXEL_BYTES: usize = 2;
const RG16_TEXEL_BYTES: usize = 4;
const RG8_TEXEL_BYTES: usize = 2;
const R32_FLOAT_TEXEL_BYTES: usize = 4;
const VK_FORMAT_R8G8_UNORM: u32 = 16;
const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
const VK_FORMAT_R8G8B8A8_SRGB: u32 = 43;
const VK_FORMAT_R16_UNORM: u32 = 70;
const VK_FORMAT_R16G16_UNORM: u32 = 77;
const VK_FORMAT_R16G16B16A16_SFLOAT: u32 = 97;
const VK_FORMAT_R32_SFLOAT: u32 = 100;
const VK_FORMAT_BC4_UNORM_BLOCK: u32 = 139;
const VK_FORMAT_BC5_UNORM_BLOCK: u32 = 141;
const VK_FORMAT_BC7_UNORM_BLOCK: u32 = 145;
const VK_FORMAT_BC7_SRGB_BLOCK: u32 = 146;

/// The layout of a texture's texels, matching a Vulkan format of the same name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureFormat {
    Bc4Unorm,
    Bc5Unorm,
    Bc7Unorm,
    Bc7Srgb,
    Rg8Unorm,
    Rgba8Unorm,
    Rgba8Srgb,
    Rgba16Float,
    R16Unorm,
    Rg16Unorm,
    R32Float,
}

impl TextureFormat {
    const ALL: [TextureFormat; 11] = [
        TextureFormat::Bc4Unorm,
        TextureFormat::Bc5Unorm,
        TextureFormat::Bc7Unorm,
        TextureFormat::Bc7Srgb,
        TextureFormat::Rg8Unorm,
        TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8Srgb,
        TextureFormat::Rgba16Float,
        TextureFormat::R16Unorm,
        TextureFormat::Rg16Unorm,
        TextureFormat::R32Float,
    ];

    /// The format a `VkFormat` value names, if it is one of these.
    pub fn from_vulkan_format(vulkan_format: u32) -> Option<TextureFormat> {
        TextureFormat::ALL
            .into_iter()
            .find(|format| format.vulkan_format() == vulkan_format)
    }

    /// The `VkFormat` value of the Vulkan format of the same name.
    pub fn vulkan_format(self) -> u32 {
        match self {
            TextureFormat::Bc4Unorm => VK_FORMAT_BC4_UNORM_BLOCK,
            TextureFormat::Bc5Unorm => VK_FORMAT_BC5_UNORM_BLOCK,
            TextureFormat::Bc7Unorm => VK_FORMAT_BC7_UNORM_BLOCK,
            TextureFormat::Bc7Srgb => VK_FORMAT_BC7_SRGB_BLOCK,
            TextureFormat::Rg8Unorm => VK_FORMAT_R8G8_UNORM,
            TextureFormat::Rgba8Unorm => VK_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::Rgba8Srgb => VK_FORMAT_R8G8B8A8_SRGB,
            TextureFormat::Rgba16Float => VK_FORMAT_R16G16B16A16_SFLOAT,
            TextureFormat::R16Unorm => VK_FORMAT_R16_UNORM,
            TextureFormat::Rg16Unorm => VK_FORMAT_R16G16_UNORM,
            TextureFormat::R32Float => VK_FORMAT_R32_SFLOAT,
        }
    }

    /// The size in bytes of one mip level of the given size.
    pub fn level_size(self, width: u32, height: u32) -> usize {
        self.checked_level_size(width, height)
            .expect("a mip level's size overflows usize")
    }

    /// The size in bytes of one mip level of the given size, or `None` when it overflows `usize`.
    pub fn checked_level_size(self, width: u32, height: u32) -> Option<usize> {
        let extent = self.block_extent();
        let columns = width.max(1).div_ceil(extent) as usize;
        let rows = height.max(1).div_ceil(extent) as usize;

        columns.checked_mul(rows)?.checked_mul(self.block_size())
    }

    /// The width and height, in texels, of the smallest unit the format stores: a compressed block, or one texel.
    pub fn block_extent(self) -> u32 {
        if self.is_block_compressed() {
            COMPRESSED_BLOCK_EXTENT
        } else {
            1
        }
    }

    /// Whether the format stores compressed blocks rather than texels.
    pub fn is_block_compressed(self) -> bool {
        match self {
            TextureFormat::Bc4Unorm | TextureFormat::Bc5Unorm | TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb => {
                true
            }
            TextureFormat::Rg8Unorm
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8Srgb
            | TextureFormat::Rgba16Float
            | TextureFormat::R16Unorm
            | TextureFormat::Rg16Unorm
            | TextureFormat::R32Float => false,
        }
    }

    /// The size in bytes of one block.
    pub fn block_size(self) -> usize {
        match self {
            TextureFormat::Bc4Unorm => BC4_BLOCK_BYTES,
            TextureFormat::Bc5Unorm | TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb => BC5_BC7_BLOCK_BYTES,
            TextureFormat::Rg8Unorm => RG8_TEXEL_BYTES,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb => RGBA8_TEXEL_BYTES,
            TextureFormat::Rgba16Float => RGBA16_FLOAT_TEXEL_BYTES,
            TextureFormat::R16Unorm => R16_TEXEL_BYTES,
            TextureFormat::Rg16Unorm => RG16_TEXEL_BYTES,
            TextureFormat::R32Float => R32_FLOAT_TEXEL_BYTES,
        }
    }

    // The color space comes from how a texture is used, not from its file: a color texture samples sRGB-decoded, and
    // formats that hold data cannot.
    pub(super) fn in_color_space(self, color_space: ColorSpace) -> anyhow::Result<TextureFormat> {
        Ok(match (self, color_space) {
            (TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb, ColorSpace::Srgb) => TextureFormat::Bc7Srgb,
            (TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb, ColorSpace::Linear) => TextureFormat::Bc7Unorm,
            (TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb, ColorSpace::Srgb) => TextureFormat::Rgba8Srgb,
            (TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb, ColorSpace::Linear) => TextureFormat::Rgba8Unorm,
            (TextureFormat::Rgba16Float, _) => TextureFormat::Rgba16Float,
            (format, ColorSpace::Linear) => format,
            (format, ColorSpace::Srgb) => bail!("{format:?} holds data, not colors"),
        })
    }
}

impl Display for TextureFormat {
    /// The Vulkan format's name, without its `VK_FORMAT_` prefix.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            TextureFormat::Bc4Unorm => "BC4_UNORM_BLOCK",
            TextureFormat::Bc5Unorm => "BC5_UNORM_BLOCK",
            TextureFormat::Bc7Unorm => "BC7_UNORM_BLOCK",
            TextureFormat::Bc7Srgb => "BC7_SRGB_BLOCK",
            TextureFormat::Rg8Unorm => "R8G8_UNORM",
            TextureFormat::Rgba8Unorm => "R8G8B8A8_UNORM",
            TextureFormat::Rgba8Srgb => "R8G8B8A8_SRGB",
            TextureFormat::Rgba16Float => "R16G16B16A16_SFLOAT",
            TextureFormat::R16Unorm => "R16_UNORM",
            TextureFormat::Rg16Unorm => "R16G16_UNORM",
            TextureFormat::R32Float => "R32_SFLOAT",
        })
    }
}
