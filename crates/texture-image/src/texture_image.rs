use std::path::Path;

use anyhow::{Context, bail, ensure};

use crate::half_float::half_to_f32;
use crate::{ColorSpace, DdsLoader, Ktx2Loader, PngLoader, TextureFormat, TextureLoader};

const HALF_BYTES: usize = 2;
const RGBA16_FLOAT_TEXEL_BYTES: usize = 4 * HALF_BYTES;

/// A decoded texture: one or more layers (six for a cube map), each holding its full mip chain, largest level first.
pub struct TextureImage {
    pub(crate) format: TextureFormat,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) mip_levels: u32,
    pub(crate) layers: Vec<Vec<u8>>,
}

impl TextureImage {
    /// Reads a texture file, choosing its loader from the file's extension.
    pub fn read(path: &Path, color_space: ColorSpace) -> anyhow::Result<TextureImage> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        let image = match extension {
            Ktx2Loader::EXTENSION => Ktx2Loader::load(&bytes, color_space),
            DdsLoader::EXTENSION => DdsLoader::load(&bytes, color_space),
            PngLoader::EXTENSION => PngLoader::load(&bytes, color_space),
            _ => bail!("{}: not a texture file", path.display()),
        };

        image.with_context(|| format!("{}", path.display()))
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    /// Each layer's mip chain, largest level first, levels packed without padding.
    pub fn layers(&self) -> &[Vec<u8>] {
        &self.layers
    }

    /// The top mip level of a half-float image, row by row, as red green blue.
    pub fn rgb_texels(&self) -> anyhow::Result<Vec<[f32; 3]>> {
        ensure!(
            self.format == TextureFormat::Rgba16Float,
            "only half-float images can be read back as texels"
        );
        let texel_count = (self.width * self.height) as usize;
        let channel = |texel: &[u8; RGBA16_FLOAT_TEXEL_BYTES], index: usize| {
            half_to_f32(u16::from_le_bytes([
                texel[index * HALF_BYTES],
                texel[index * HALF_BYTES + 1],
            ]))
        };

        Ok(self.layers[0]
            .as_chunks::<RGBA16_FLOAT_TEXEL_BYTES>()
            .0
            .iter()
            .take(texel_count)
            .map(|texel| [channel(texel, 0), channel(texel, 1), channel(texel, 2)])
            .collect())
    }

    pub(crate) fn layer_size(&self) -> usize {
        (0..self.mip_levels)
            .map(|level| self.format.level_size(self.width >> level, self.height >> level))
            .sum()
    }
}
