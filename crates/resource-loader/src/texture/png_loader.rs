use image::ImageFormat;

use super::{ColorSpace, TextureFormat, TextureImage, TextureLoader};

/// Reads PNG files as a single RGBA level.
pub struct PngLoader;

impl TextureLoader for PngLoader {
    const EXTENSION: &'static str = "png";

    fn load(bytes: &[u8], color_space: ColorSpace) -> anyhow::Result<TextureImage> {
        let rgba = image::load_from_memory_with_format(bytes, ImageFormat::Png)?.into_rgba8();

        Ok(TextureImage {
            format: TextureFormat::Rgba8Unorm.in_color_space(color_space)?,
            width: rgba.width(),
            height: rgba.height(),
            mip_levels: 1,
            layers: vec![rgba.into_raw()],
        })
    }
}
