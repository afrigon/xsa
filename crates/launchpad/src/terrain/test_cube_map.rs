use resource_loader::texture::{Ktx2Loader, TextureFormat, TextureLoader};
use resource_loader::utils::CubeFace;

use super::CubeMapInput;

const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
const KTX2_HEADER_SIZE: usize = 80;
const KTX2_LEVEL_ENTRY_SIZE: usize = 24;
const KTX2_TYPE_SIZE: u32 = 1;

/// The shape of an uncompressed KTX2 cube map built in memory, its texels given by a function of the face, the mip
/// level's extent and the texel's column and row.
pub struct TestCubeMap {
    pub format: TextureFormat,
    pub extent: u32,
    pub mip_levels: u32,
}

impl TestCubeMap {
    pub fn full_chain(format: TextureFormat, extent: u32) -> TestCubeMap {
        TestCubeMap {
            format,
            extent,
            mip_levels: extent.ilog2() + 1,
        }
    }

    pub fn input(&self, texel: impl Fn(CubeFace, u32, u32, u32) -> Vec<u8>) -> anyhow::Result<CubeMapInput> {
        CubeMapInput::new(Ktx2Loader::load(&self.bytes(texel))?)
    }

    pub fn bytes(&self, texel: impl Fn(CubeFace, u32, u32, u32) -> Vec<u8>) -> Vec<u8> {
        let levels: Vec<Vec<u8>> = (0..self.mip_levels)
            .map(|mip| {
                let extent = (self.extent >> mip).max(1);

                CubeFace::ALL
                    .into_iter()
                    .flat_map(|face| (0..extent * extent).map(move |index| (face, index)))
                    .flat_map(|(face, index)| texel(face, extent, index % extent, index / extent))
                    .collect()
            })
            .collect();
        let mut bytes = KTX2_IDENTIFIER.to_vec();
        let header = [
            self.format.vulkan_format().value,
            KTX2_TYPE_SIZE,
            self.extent,
            self.extent,
            0,
            0,
            CubeFace::ALL.len() as u32,
            self.mip_levels,
            0,
        ];

        for word in header {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        bytes.resize(KTX2_HEADER_SIZE, 0);
        let mut offset = KTX2_HEADER_SIZE + levels.len() * KTX2_LEVEL_ENTRY_SIZE;

        for level in &levels {
            for long in [offset, level.len(), level.len()] {
                bytes.extend_from_slice(&(long as u64).to_le_bytes());
            }

            offset += level.len();
        }

        for level in &levels {
            bytes.extend_from_slice(level);
        }

        bytes
    }
}
