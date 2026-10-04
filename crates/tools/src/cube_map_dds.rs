use std::fs::File;
use std::io::{BufWriter, Write};

use crate::cube_face::{CubeFace, FACE_SIZE};

const MAGIC: &[u8; 4] = b"DDS ";
const HEADER_SIZE: u32 = 124;
const HEADER_RESERVED_BYTES: usize = 44;
const PIXEL_FORMAT_SIZE: u32 = 32;
const PIXEL_FORMAT_MASK_BYTES: usize = 20;
const DX10_FOURCC: &[u8; 4] = b"DX10";
const BYTES_PER_TEXEL: u32 = 4;
const OPAQUE_ALPHA: u8 = 255;

const DDSD_CAPS: u32 = 0x1;
const DDSD_HEIGHT: u32 = 0x2;
const DDSD_WIDTH: u32 = 0x4;
const DDSD_PITCH: u32 = 0x8;
const DDSD_PIXELFORMAT: u32 = 0x1000;
const DDSD_MIPMAPCOUNT: u32 = 0x20000;
const DDPF_FOURCC: u32 = 0x4;
const DDSCAPS_COMPLEX: u32 = 0x8;
const DDSCAPS_TEXTURE: u32 = 0x1000;
const DDSCAPS_MIPMAP: u32 = 0x400000;
const DDSCAPS2_CUBEMAP: u32 = 0x200;
const DDSCAPS2_CUBEMAP_ALL_FACES: u32 = 0xFC00;
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
const D3D10_RESOURCE_DIMENSION_TEXTURE2D: u32 = 3;
const D3D10_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
const DX10_ARRAY_SIZE: u32 = 1;

const SRGB_LINEAR_THRESHOLD: f32 = 0.003_130_8;
const SRGB_LINEAR_SCALE: f32 = 12.92;
const SRGB_GAMMA: f32 = 2.4;
const SRGB_SCALE: f32 = 1.055;
const SRGB_OFFSET: f32 = 0.055;

pub struct CubeMapDds {
    pub mip_chains: Vec<Vec<CubeFace>>,
}

impl CubeMapDds {
    pub fn write(&self, path: &str, exposure: f32) -> anyhow::Result<()> {
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(&self.header())?;

        for chain in &self.mip_chains {
            for level in chain {
                let bytes: Vec<u8> = level
                    .texels
                    .iter()
                    .flat_map(|color| {
                        let [red, green, blue] = (*color * exposure).to_array().map(CubeMapDds::encode_srgb);
                        [red, green, blue, OPAQUE_ALPHA]
                    })
                    .collect();
                file.write_all(&bytes)?;
            }
        }

        file.flush()?;

        Ok(())
    }

    fn header(&self) -> Vec<u8> {
        let mip_count = self.mip_chains[0].len() as u32;
        let size = FACE_SIZE as u32;
        let mut header = Vec::new();
        let words = |header: &mut Vec<u8>, values: &[u32]| {
            for value in values {
                header.extend_from_slice(&value.to_le_bytes());
            }
        };

        header.extend_from_slice(MAGIC);
        words(
            &mut header,
            &[
                HEADER_SIZE,
                DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PITCH | DDSD_PIXELFORMAT | DDSD_MIPMAPCOUNT,
                size,
                size,
                size * BYTES_PER_TEXEL,
                0,
                mip_count,
            ],
        );
        header.extend_from_slice(&[0; HEADER_RESERVED_BYTES]);
        words(&mut header, &[PIXEL_FORMAT_SIZE, DDPF_FOURCC]);
        header.extend_from_slice(DX10_FOURCC);
        header.extend_from_slice(&[0; PIXEL_FORMAT_MASK_BYTES]);
        words(
            &mut header,
            &[
                DDSCAPS_TEXTURE | DDSCAPS_COMPLEX | DDSCAPS_MIPMAP,
                DDSCAPS2_CUBEMAP | DDSCAPS2_CUBEMAP_ALL_FACES,
                0,
                0,
                0,
            ],
        );
        words(
            &mut header,
            &[
                DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                D3D10_RESOURCE_DIMENSION_TEXTURE2D,
                D3D10_RESOURCE_MISC_TEXTURECUBE,
                DX10_ARRAY_SIZE,
                0,
            ],
        );

        header
    }

    fn encode_srgb(linear: f32) -> u8 {
        let linear = linear.clamp(0.0, 1.0);
        let encoded = if linear <= SRGB_LINEAR_THRESHOLD {
            linear * SRGB_LINEAR_SCALE
        } else {
            SRGB_SCALE * linear.powf(1.0 / SRGB_GAMMA) - SRGB_OFFSET
        };

        (encoded * f32::from(u8::MAX) + 0.5) as u8
    }
}
