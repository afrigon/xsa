use image::ImageFormat;

use super::{TextureFormat, TextureImage, TextureLoader};

/// Reads PNG files as a single sRGB-encoded RGBA level, PNG's default color space.
pub struct PngLoader;

impl TextureLoader for PngLoader {
    const EXTENSION: &'static str = "png";

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage> {
        let rgba = image::load_from_memory_with_format(bytes, ImageFormat::Png)?.into_rgba8();

        Ok(TextureImage {
            format: TextureFormat::Rgba8Srgb,
            width: rgba.width(),
            height: rgba.height(),
            mip_levels: 1,
            layers: vec![rgba.into_raw()],
        })
    }
}
