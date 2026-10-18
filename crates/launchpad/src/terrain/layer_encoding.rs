use anyhow::{Context, bail};
use block_compression::{BC7Settings, CompressionVariant};
use resource_loader::terrain::{HeightRange, PAGE_EXTENT, PageEncoding};
use resource_loader::texture::{HalfFloat, TextureFormat};

use super::EncodedPage;

const RGBA8_CHANNELS: usize = 4;
const RG8_CHANNELS: usize = 2;
const UNORM8_MAXIMUM: f32 = u8::MAX as f32;
const OPAQUE: u8 = u8::MAX;
const SRGB_LINEAR_THRESHOLD: f32 = 0.040_45;
const LINEAR_SRGB_THRESHOLD: f32 = 0.003_130_8;
const SRGB_LINEAR_SLOPE: f32 = 12.92;
const SRGB_OFFSET: f32 = 0.055;
const SRGB_GAMMA: f32 = 2.4;

/// How a terrain layer's input texels become its pages, chosen by `--height` and the input's texel format.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerEncoding {
    /// `R32_SFLOAT` metres into `R16_UNORM` over each page's own range.
    Heights,
    /// `R8G8B8A8_SRGB` into `BC7_SRGB_BLOCK`.
    Bc7,
    /// `R8G8_UNORM` into `BC5_UNORM_BLOCK`.
    Bc5,
    /// `R16G16B16A16_SFLOAT`, stored as is.
    Rgba16Float,
}

impl LayerEncoding {
    const ALL: [LayerEncoding; 4] = [
        LayerEncoding::Heights,
        LayerEncoding::Bc7,
        LayerEncoding::Bc5,
        LayerEncoding::Rgba16Float,
    ];

    pub fn new(encoding: PageEncoding, input: TextureFormat) -> anyhow::Result<LayerEncoding> {
        Ok(match (encoding, input) {
            (PageEncoding::PageHeights, TextureFormat::R32Float) => LayerEncoding::Heights,
            (PageEncoding::PageHeights, other) => {
                bail!("{other} texels: heights (--height) must be R32_SFLOAT metres relative to the body's radius")
            }
            (PageEncoding::Direct, TextureFormat::Rgba8Srgb) => LayerEncoding::Bc7,
            (PageEncoding::Direct, TextureFormat::Rg8Unorm) => LayerEncoding::Bc5,
            (PageEncoding::Direct, TextureFormat::Rgba16Float) => LayerEncoding::Rgba16Float,
            (PageEncoding::Direct, TextureFormat::R32Float) => {
                bail!("R32_SFLOAT texels are heights: convert them with --height")
            }
            (PageEncoding::Direct, other) => bail!(
                "{other} texels: a terrain layer is R8G8B8A8_SRGB, R8G8_UNORM or R16G16B16A16_SFLOAT, or \
                 R32_SFLOAT heights with --height"
            ),
        })
    }

    /// The layer encoding whose pages a `.terrain` file of that format and encoding holds.
    pub fn from_page_format(format: TextureFormat, encoding: PageEncoding) -> anyhow::Result<LayerEncoding> {
        LayerEncoding::ALL
            .into_iter()
            .find(|layer| layer.page_format() == format && layer.page_encoding() == encoding)
            .with_context(|| format!("no terrain layer stores {format} pages with this encoding"))
    }

    pub fn input_format(self) -> TextureFormat {
        match self {
            LayerEncoding::Heights => TextureFormat::R32Float,
            LayerEncoding::Bc7 => TextureFormat::Rgba8Srgb,
            LayerEncoding::Bc5 => TextureFormat::Rg8Unorm,
            LayerEncoding::Rgba16Float => TextureFormat::Rgba16Float,
        }
    }

    pub fn page_format(self) -> TextureFormat {
        match self {
            LayerEncoding::Heights => TextureFormat::R16Unorm,
            LayerEncoding::Bc7 => TextureFormat::Bc7Srgb,
            LayerEncoding::Bc5 => TextureFormat::Bc5Unorm,
            LayerEncoding::Rgba16Float => TextureFormat::Rgba16Float,
        }
    }

    pub fn page_encoding(self) -> PageEncoding {
        match self {
            LayerEncoding::Heights => PageEncoding::PageHeights,
            LayerEncoding::Bc7 | LayerEncoding::Bc5 | LayerEncoding::Rgba16Float => PageEncoding::Direct,
        }
    }

    /// Encodes a page's input texels, rows top to bottom, into the page format, with its height range for heights.
    pub fn encode(self, texels: &[u8]) -> EncodedPage {
        match self {
            LayerEncoding::Heights => {
                let heights = texels
                    .as_chunks::<{ size_of::<f32>() }>()
                    .0
                    .iter()
                    .map(|bytes| f32::from_le_bytes(*bytes));
                let range = HeightRange::spanning(heights.clone()).unwrap();

                EncodedPage {
                    data: heights.flat_map(|height| range.value(height).to_le_bytes()).collect(),
                    range: Some(range),
                }
            }
            LayerEncoding::Bc7 => EncodedPage {
                data: LayerEncoding::compressed(CompressionVariant::BC7(BC7Settings::alpha_slow()), texels),
                range: None,
            },
            LayerEncoding::Bc5 => {
                let rgba: Vec<u8> = texels
                    .as_chunks::<RG8_CHANNELS>()
                    .0
                    .iter()
                    .flat_map(|[red, green]| [*red, *green, 0, OPAQUE])
                    .collect();

                EncodedPage {
                    data: LayerEncoding::compressed(CompressionVariant::BC5, &rgba),
                    range: None,
                }
            }
            LayerEncoding::Rgba16Float => EncodedPage {
                data: texels.to_vec(),
                range: None,
            },
        }
    }

    /// Decodes a page in the page format back into input texels, rows top to bottom; `range` is its entry's.
    pub fn decode(self, page: &[u8], range: Option<HeightRange>) -> Vec<u8> {
        match self {
            LayerEncoding::Heights => {
                let range = range.expect("the reader refuses height pages without their range");

                page.as_chunks::<{ size_of::<u16>() }>()
                    .0
                    .iter()
                    .flat_map(|value| range.height(u16::from_le_bytes(*value)).to_le_bytes())
                    .collect()
            }
            LayerEncoding::Bc7 => LayerEncoding::decompressed(CompressionVariant::BC7(BC7Settings::alpha_slow()), page),
            LayerEncoding::Bc5 => LayerEncoding::decompressed(CompressionVariant::BC5, page)
                .as_chunks::<RGBA8_CHANNELS>()
                .0
                .iter()
                .flat_map(|[red, green, _, _]| [*red, *green])
                .collect(),
            LayerEncoding::Rgba16Float => page.to_vec(),
        }
    }

    /// The input texel standing for an absent page: transparent, or NaN for heights, or 0 for normals.
    pub fn absent_texel(self) -> Vec<u8> {
        match self {
            LayerEncoding::Heights => f32::NAN.to_le_bytes().to_vec(),
            LayerEncoding::Bc7 => vec![0; RGBA8_CHANNELS],
            LayerEncoding::Bc5 => vec![0; RG8_CHANNELS],
            LayerEncoding::Rgba16Float => vec![0; TextureFormat::Rgba16Float.block_size()],
        }
    }

    /// The average of several input texels, written into `output`: colors in linear light, everything else as stored.
    pub fn average(self, texels: &[&[u8]], output: &mut [u8]) {
        let count = texels.len() as f32;

        match self {
            LayerEncoding::Heights => {
                let sum: f32 = texels
                    .iter()
                    .map(|texel| f32::from_le_bytes((*texel).try_into().unwrap()))
                    .sum();
                output.copy_from_slice(&(sum / count).to_le_bytes());
            }
            LayerEncoding::Bc7 => {
                for channel in 0..RGBA8_CHANNELS {
                    let values = texels.iter().map(|texel| f32::from(texel[channel]) / UNORM8_MAXIMUM);
                    let is_color = channel < RGBA8_CHANNELS - 1;
                    let average = if is_color {
                        linear_to_srgb(values.map(srgb_to_linear).sum::<f32>() / count)
                    } else {
                        values.sum::<f32>() / count
                    };
                    output[channel] = (average * UNORM8_MAXIMUM).round() as u8;
                }
            }
            LayerEncoding::Bc5 => {
                for channel in 0..RG8_CHANNELS {
                    let sum: f32 = texels.iter().map(|texel| f32::from(texel[channel])).sum();
                    output[channel] = (sum / count).round() as u8;
                }
            }
            LayerEncoding::Rgba16Float => {
                for (channel, half) in output.as_chunks_mut::<{ size_of::<u16>() }>().0.iter_mut().enumerate() {
                    let sum: f32 = texels
                        .iter()
                        .map(|texel| {
                            let offset = channel * size_of::<u16>();
                            HalfFloat {
                                bits: u16::from_le_bytes([texel[offset], texel[offset + 1]]),
                            }
                            .to_f32()
                        })
                        .sum();
                    *half = HalfFloat::from_f32(sum / count).bits.to_le_bytes();
                }
            }
        }
    }

    fn compressed(variant: CompressionVariant, rgba: &[u8]) -> Vec<u8> {
        let mut blocks = vec![0; variant.blocks_byte_size(PAGE_EXTENT, PAGE_EXTENT)];
        block_compression::encode::compress_rgba8(
            variant,
            rgba,
            &mut blocks,
            PAGE_EXTENT,
            PAGE_EXTENT,
            PAGE_EXTENT * RGBA8_CHANNELS as u32,
        );

        blocks
    }

    fn decompressed(variant: CompressionVariant, blocks: &[u8]) -> Vec<u8> {
        let mut rgba = vec![0; (PAGE_EXTENT * PAGE_EXTENT) as usize * RGBA8_CHANNELS];
        block_compression::decode::decompress_blocks_as_rgba8(variant, PAGE_EXTENT, PAGE_EXTENT, blocks, &mut rgba);

        rgba
    }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= SRGB_LINEAR_THRESHOLD {
        value / SRGB_LINEAR_SLOPE
    } else {
        ((value + SRGB_OFFSET) / (1.0 + SRGB_OFFSET)).powf(SRGB_GAMMA)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= LINEAR_SRGB_THRESHOLD {
        value * SRGB_LINEAR_SLOPE
    } else {
        (1.0 + SRGB_OFFSET) * value.powf(1.0 / SRGB_GAMMA) - SRGB_OFFSET
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs_matching_no_layer_are_refused() {
        assert!(LayerEncoding::new(PageEncoding::Direct, TextureFormat::Rgba8Unorm).is_err());
        assert!(LayerEncoding::new(PageEncoding::Direct, TextureFormat::R32Float).is_err());
        assert!(LayerEncoding::new(PageEncoding::PageHeights, TextureFormat::R16Unorm).is_err());
        assert!(LayerEncoding::new(PageEncoding::PageHeights, TextureFormat::Rgba8Srgb).is_err());
    }

    #[test]
    fn each_page_format_leads_back_to_its_layer() {
        for layer in LayerEncoding::ALL {
            assert_eq!(
                LayerEncoding::from_page_format(layer.page_format(), layer.page_encoding()).unwrap(),
                layer
            );
        }

        assert!(LayerEncoding::from_page_format(TextureFormat::R16Unorm, PageEncoding::Direct).is_err());
    }

    #[test]
    fn heights_decode_back_through_their_page_range() {
        let range = HeightRange {
            minimum: -5.0,
            maximum: 20.0,
        };
        let page: Vec<u8> = [0u16, u16::MAX].iter().flat_map(|value| value.to_le_bytes()).collect();
        let heights: Vec<u8> = [-5.0f32, 20.0].iter().flat_map(|height| height.to_le_bytes()).collect();

        assert_eq!(LayerEncoding::Heights.decode(&page, Some(range)), heights);
    }

    #[test]
    fn colours_average_in_linear_light() {
        let mut output = [0; RGBA8_CHANNELS];
        LayerEncoding::Bc7.average(&[&[0, 0, 0, 0], &[255, 255, 255, 255], &[0, 0, 0, 255]], &mut output);

        assert_eq!(output, [156, 156, 156, 170]);
    }

    #[test]
    fn half_floats_average_as_their_values() {
        let texel = |value: f32| HalfFloat::from_f32(value).bits.to_le_bytes().repeat(4);
        let mut output = [0; 8];
        LayerEncoding::Rgba16Float.average(&[&texel(1.0), &texel(2.0), &texel(6.0)], &mut output);

        assert_eq!(output.as_slice(), texel(3.0));
    }

    #[test]
    fn a_page_of_heights_spans_its_own_range() {
        let texels: Vec<u8> = [-5.0f32, 20.0, 7.5]
            .iter()
            .flat_map(|height| height.to_le_bytes())
            .collect();
        let page = LayerEncoding::Heights.encode(&texels);

        assert_eq!(
            page.range,
            Some(HeightRange {
                minimum: -5.0,
                maximum: 20.0
            })
        );
        assert_eq!(
            page.data,
            [0u16, u16::MAX, 32768]
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>()
        );
    }
}
