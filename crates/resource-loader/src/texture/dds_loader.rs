use std::collections::BTreeMap;

use anyhow::{bail, ensure};

use super::{TextureFormat, TextureImage, TextureLoader, TextureShape};

const DDS_MAGIC: &[u8; 4] = b"DDS ";
const DX10_FOURCC: &[u8; 4] = b"DX10";
const DDS_HEADER_SIZE: usize = 148;
const HEIGHT_OFFSET: usize = 12;
const WIDTH_OFFSET: usize = 16;
const MIP_COUNT_OFFSET: usize = 28;
const FOURCC_OFFSET: usize = 84;
const DXGI_FORMAT_OFFSET: usize = 128;
const MISC_FLAGS_OFFSET: usize = 136;
const WORD_BYTES: usize = 4;
const DXGI_FORMAT_R16G16B16A16_FLOAT: u32 = 10;
const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
const DXGI_FORMAT_R16G16_UNORM: u32 = 35;
const DXGI_FORMAT_R16_UNORM: u32 = 56;
const DXGI_FORMAT_BC4_UNORM: u32 = 80;
const DXGI_FORMAT_BC5_UNORM: u32 = 83;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;
const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
const CUBE_FACE_COUNT: usize = 6;

/// Reads DDS files with a DX10 header: the layers one after another, each with its full mip chain.
pub struct DdsLoader;

impl TextureLoader for DdsLoader {
    const EXTENSION: &'static str = "dds";

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage> {
        ensure!(
            bytes.len() >= DDS_HEADER_SIZE && &bytes[..DDS_MAGIC.len()] == DDS_MAGIC,
            "not a DDS file"
        );
        ensure!(
            &bytes[FOURCC_OFFSET..FOURCC_OFFSET + DX10_FOURCC.len()] == DX10_FOURCC,
            "only DDS files with a DX10 header are supported"
        );

        let word = |offset: usize| u32::from_le_bytes(bytes[offset..offset + WORD_BYTES].try_into().unwrap());
        let format = match word(DXGI_FORMAT_OFFSET) {
            DXGI_FORMAT_R16G16B16A16_FLOAT => TextureFormat::Rgba16Float,
            DXGI_FORMAT_R8G8B8A8_UNORM => TextureFormat::Rgba8Unorm,
            DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => TextureFormat::Rgba8Srgb,
            DXGI_FORMAT_R16G16_UNORM => TextureFormat::Rg16Unorm,
            DXGI_FORMAT_R16_UNORM => TextureFormat::R16Unorm,
            DXGI_FORMAT_BC4_UNORM => TextureFormat::Bc4Unorm,
            DXGI_FORMAT_BC5_UNORM => TextureFormat::Bc5Unorm,
            DXGI_FORMAT_BC7_UNORM => TextureFormat::Bc7Unorm,
            DXGI_FORMAT_BC7_UNORM_SRGB => TextureFormat::Bc7Srgb,
            other => bail!("unsupported DXGI format {other}"),
        };
        let (shape, layer_count) = if word(MISC_FLAGS_OFFSET) & DDS_RESOURCE_MISC_TEXTURECUBE != 0 {
            (TextureShape::CubeMap, CUBE_FACE_COUNT)
        } else {
            (TextureShape::Plane, 1)
        };
        let mut image = TextureImage {
            format,
            shape,
            width: word(WIDTH_OFFSET),
            height: word(HEIGHT_OFFSET),
            mip_levels: word(MIP_COUNT_OFFSET).max(1),
            layers: Vec::new(),
            metadata: BTreeMap::new(),
        };
        let layer_size = image.layer_size()?;
        let data = &bytes[DDS_HEADER_SIZE..];
        ensure!(
            layer_size
                .checked_mul(layer_count)
                .is_some_and(|size| data.len() >= size),
            "the file is truncated"
        );

        image.layers = data
            .chunks_exact(layer_size)
            .take(layer_count)
            .map(<[u8]>::to_vec)
            .collect();

        Ok(image)
    }
}
