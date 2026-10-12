use anyhow::bail;

use crate::ColorSpace;

const BLOCK_SIZE: usize = 4;
const BC4_BLOCK_BYTES: usize = 8;
const BC5_BC7_BLOCK_BYTES: usize = 16;
const RGBA8_TEXEL_BYTES: usize = 4;
const RGBA16_FLOAT_TEXEL_BYTES: usize = 8;
const R16_TEXEL_BYTES: usize = 2;
const RG16_TEXEL_BYTES: usize = 4;

/// The layout of a texture's texels, matching a Vulkan format of the same name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureFormat {
    Bc4Unorm,
    Bc5Unorm,
    Bc7Unorm,
    Bc7Srgb,
    Rgba8Unorm,
    Rgba8Srgb,
    Rgba16Float,
    R16Unorm,
    Rg16Unorm,
}

impl TextureFormat {
    /// The size in bytes of one mip level of the given size.
    pub fn level_size(self, width: u32, height: u32) -> usize {
        let width = width.max(1) as usize;
        let height = height.max(1) as usize;
        let blocks = width.div_ceil(BLOCK_SIZE) * height.div_ceil(BLOCK_SIZE);

        match self {
            TextureFormat::Bc4Unorm => blocks * BC4_BLOCK_BYTES,
            TextureFormat::Bc5Unorm | TextureFormat::Bc7Unorm | TextureFormat::Bc7Srgb => blocks * BC5_BC7_BLOCK_BYTES,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Srgb => width * height * RGBA8_TEXEL_BYTES,
            TextureFormat::Rgba16Float => width * height * RGBA16_FLOAT_TEXEL_BYTES,
            TextureFormat::R16Unorm => width * height * R16_TEXEL_BYTES,
            TextureFormat::Rg16Unorm => width * height * RG16_TEXEL_BYTES,
        }
    }

    // The color space comes from how a texture is used, not from its file: a color texture samples sRGB-decoded, and
    // formats that hold data cannot.
    pub(crate) fn in_color_space(self, color_space: ColorSpace) -> anyhow::Result<TextureFormat> {
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
