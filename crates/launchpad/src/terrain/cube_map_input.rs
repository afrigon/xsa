use std::path::Path;

use anyhow::{Context, ensure};
use resource_loader::terrain::{CubeFace, PAGE_INTERIOR};
use resource_loader::texture::{TextureFormat, TextureImage};

/// A terrain layer's cube map, checked against what the conversion needs: six square power-of-two faces of at least
/// one page, with their full mip chain, uncompressed.
pub struct CubeMapInput {
    format: TextureFormat,
    face_extent: u32,
    level_count: u32,
    faces: Vec<Vec<u8>>,
}

impl CubeMapInput {
    pub fn read(path: &Path) -> anyhow::Result<CubeMapInput> {
        let image = TextureImage::read_stored(path)?;
        CubeMapInput::new(image).with_context(|| format!("{}", path.display()))
    }

    pub fn new(image: TextureImage) -> anyhow::Result<CubeMapInput> {
        let format = image.format();
        let face_extent = image.width();
        ensure!(
            !format.is_block_compressed(),
            "{format} texels: the input must be uncompressed, this command compresses each page itself"
        );
        ensure!(
            image.layers().len() == CubeFace::ALL.len(),
            "{} faces: a cube map has {}",
            image.layers().len(),
            CubeFace::ALL.len()
        );
        ensure!(
            image.width() == image.height(),
            "{}×{} faces: they must be square",
            image.width(),
            image.height()
        );
        ensure!(
            face_extent.is_power_of_two() && face_extent >= PAGE_INTERIOR,
            "{face_extent}-texel faces: the size must be a power of two of at least {PAGE_INTERIOR}"
        );

        let full_chain = face_extent.ilog2() + 1;
        ensure!(
            image.mip_levels() == full_chain,
            "{} mip levels: {face_extent}-texel faces need their full chain of {full_chain}",
            image.mip_levels()
        );

        Ok(CubeMapInput {
            format,
            face_extent,
            level_count: (face_extent / PAGE_INTERIOR).ilog2() + 1,
            faces: image.into_layers(),
        })
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn face_extent(&self) -> u32 {
        self.face_extent
    }

    /// The number of terrain levels: level 0 is one page per face, the finest is the input's top mip level.
    pub fn level_count(&self) -> u32 {
        self.level_count
    }

    /// The width and height, in texels, of a face at a terrain level.
    pub fn level_extent(&self, level: u32) -> u32 {
        PAGE_INTERIOR << level
    }

    /// A face's texels at a terrain level, rows top to bottom.
    pub fn level_texels(&self, face: CubeFace, level: u32) -> &[u8] {
        let mip = self.level_count - 1 - level;
        let offset = (0..mip)
            .map(|mip| self.format.level_size(self.face_extent >> mip, self.face_extent >> mip))
            .sum::<usize>();
        let extent = self.level_extent(level);

        &self.faces[face.index()][offset..offset + self.format.level_size(extent, extent)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TestCubeMap;

    fn refusal(cube_map: TestCubeMap) -> String {
        let texel_size = cube_map.format.block_size();
        cube_map
            .input(|_, _, _, _| vec![0; texel_size])
            .err()
            .unwrap()
            .to_string()
    }

    #[test]
    fn faces_smaller_than_a_page_are_refused() {
        assert!(refusal(TestCubeMap::full_chain(TextureFormat::R32Float, 64)).contains("at least 128"));
    }

    #[test]
    fn faces_that_are_not_a_power_of_two_are_refused() {
        assert!(refusal(TestCubeMap::full_chain(TextureFormat::R32Float, 192)).contains("power of two"));
    }

    #[test]
    fn a_partial_mip_chain_is_refused() {
        let cube_map = TestCubeMap {
            format: TextureFormat::R32Float,
            extent: 256,
            mip_levels: 3,
        };

        assert!(refusal(cube_map).contains("full chain of 9"));
    }

    #[test]
    fn compressed_texels_are_refused() {
        // A BC7 block is one byte per texel, so down to the 4×4 level the builder's texels fill whole blocks.
        let cube_map = TestCubeMap {
            format: TextureFormat::Bc7Srgb,
            extent: 256,
            mip_levels: 7,
        };

        assert!(
            cube_map
                .input(|_, _, _, _| vec![0])
                .err()
                .unwrap()
                .to_string()
                .contains("uncompressed")
        );
    }
}
