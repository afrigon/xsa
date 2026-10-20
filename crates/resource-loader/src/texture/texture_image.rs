use std::path::Path;

use anyhow::{Context, bail, ensure};

use super::{
    ColorSpace, DdsLoader, Ktx2Loader, PngLoader, TextureFormat, TextureLoader, TextureMetadata, TextureShape,
};
use crate::utils::HalfFloat;

const HALF_BYTES: usize = 2;
const RGBA16_FLOAT_TEXEL_BYTES: usize = 4 * HALF_BYTES;

/// A decoded texture: one or more layers arranged as its shape says, each holding its full mip chain, largest level
/// first, with the key/value metadata its file carries.
pub struct TextureImage {
    pub(super) format: TextureFormat,
    pub(super) shape: TextureShape,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) mip_levels: u32,
    pub(super) layers: Vec<Vec<u8>>,
    pub(super) metadata: TextureMetadata,
}

impl TextureImage {
    /// A texture from its layers' texels, each layer its mip chain, largest level first, refusing layers of the wrong
    /// size or count for its shape.
    pub fn new(
        format: TextureFormat,
        shape: TextureShape,
        width: u32,
        height: u32,
        mip_levels: u32,
        layers: Vec<Vec<u8>>,
    ) -> anyhow::Result<TextureImage> {
        let image = TextureImage {
            format,
            shape,
            width,
            height,
            mip_levels,
            layers,
            metadata: TextureMetadata::default(),
        };
        let layer_size = image.layer_size()?;
        image.check_layer_count()?;

        for (index, layer) in image.layers.iter().enumerate() {
            ensure!(
                layer.len() == layer_size,
                "layer {index} holds {} bytes, expected {layer_size}",
                layer.len()
            );
        }

        Ok(image)
    }

    /// Reads a texture file, choosing its loader from the file's extension, its texels meant in a color space.
    pub fn read(path: &Path, color_space: ColorSpace) -> anyhow::Result<TextureImage> {
        TextureImage::read_stored(path)?
            .in_color_space(color_space)
            .with_context(|| format!("{}", path.display()))
    }

    /// Reads a texture file, choosing its loader from the file's extension, in the texel format the file stores.
    pub fn read_stored(path: &Path) -> anyhow::Result<TextureImage> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        let image = match extension {
            Ktx2Loader::EXTENSION => Ktx2Loader::load(&bytes),
            DdsLoader::EXTENSION => DdsLoader::load(&bytes),
            PngLoader::EXTENSION => PngLoader::load(&bytes),
            _ => bail!("{}: not a texture file", path.display()),
        };

        image.with_context(|| format!("{}", path.display()))
    }

    /// The same texels, their format reinterpreted for how they are meant: sRGB-decoded colors or linear data.
    pub fn in_color_space(self, color_space: ColorSpace) -> anyhow::Result<TextureImage> {
        Ok(TextureImage {
            format: self.format.in_color_space(color_space)?,
            ..self
        })
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn shape(&self) -> TextureShape {
        self.shape
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

    /// Each layer's mip chain, as `layers` lays them out, owned.
    pub fn into_layers(self) -> Vec<Vec<u8>> {
        self.layers
    }

    /// The value stored under a key/value metadata key, as the file holds it.
    pub fn metadata(&self, key: &str) -> Option<&[u8]> {
        self.metadata.value(key)
    }

    /// The numbers stored under a key/value metadata key, written as text separated by spaces and ending with a NUL.
    pub fn metadata_numbers(&self, key: &str) -> anyhow::Result<Vec<f32>> {
        self.metadata.numbers(key)
    }

    /// Every key/value metadata entry the file carries.
    pub fn all_metadata(&self) -> &TextureMetadata {
        &self.metadata
    }

    /// The top mip level of a half-float image, row by row, as red green blue.
    pub fn rgb_texels(&self) -> anyhow::Result<Vec<[f32; 3]>> {
        ensure!(
            self.format == TextureFormat::Rgba16Float,
            "only half-float images can be read back as texels"
        );
        let texel_count = (self.width * self.height) as usize;
        let channel = |texel: &[u8; RGBA16_FLOAT_TEXEL_BYTES], index: usize| {
            HalfFloat {
                bits: u16::from_le_bytes([texel[index * HALF_BYTES], texel[index * HALF_BYTES + 1]]),
            }
            .to_f32()
        };

        Ok(self.layers[0]
            .as_chunks::<RGBA16_FLOAT_TEXEL_BYTES>()
            .0
            .iter()
            .take(texel_count)
            .map(|texel| [channel(texel, 0), channel(texel, 1), channel(texel, 2)])
            .collect())
    }

    pub(super) fn check_layer_count(&self) -> anyhow::Result<()> {
        ensure!(
            self.shape.holds(self.layers.len()),
            "a {} cannot hold {} layers",
            self.shape,
            self.layers.len()
        );

        Ok(())
    }

    // The size in bytes of a layer's mip chain as the file's header declares it, refusing shapes no valid file has
    // and sizes that overflow, so nothing is allocated from an impossible header.
    pub(super) fn layer_size(&self) -> anyhow::Result<usize> {
        ensure!(
            self.width > 0 && self.height > 0,
            "{}×{} texels",
            self.width,
            self.height
        );

        let full_chain = self.width.max(self.height).ilog2() + 1;
        ensure!(
            self.mip_levels <= full_chain,
            "{} mip levels: a {}×{} texture has at most {full_chain}",
            self.mip_levels,
            self.width,
            self.height
        );

        (0..self.mip_levels)
            .try_fold(0usize, |size, level| {
                size.checked_add(
                    self.format
                        .checked_level_size(self.width >> level, self.height >> level)?,
                )
            })
            .with_context(|| format!("{}×{} texels are too large", self.width, self.height))
    }
}
