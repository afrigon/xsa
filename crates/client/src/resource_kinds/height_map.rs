use std::path::Path;

use anyhow::ensure;
use memory_vulkan::{BindlessTable, Image, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureFormat, TextureImage, TextureKind};
use resource_loader::utils::CubeFace;
use resource_vulkan::{LoadContext, MemoryUsage, PendingResource, PendingTexture, StreamedMips, Texture};
use xsa_core::terrain::{HeightMapLevel, SPLINE_TEXELS};

use super::{HeightBounds, HeightMapOptions};

// A body's heights: an equal-angle cube map of 16-bit values, its resident levels on the GPU as a six-layer array read
// texel by texel, and its finest resident level on the CPU, with that level's bounds. Its chain ends at the last level
// whose faces are as wide as the B-spline reads: narrower faces cannot hold the texels it reaches across an edge.
pub struct HeightMap {
    texture: Texture,
    first_level: u32,
    mip_levels: u32,
    extent: u32,
    level: HeightMapLevel,
    bounds: HeightBounds,
}

impl HeightMap {
    // Where shaders find it in the texture array binding's array.
    pub fn index(&self) -> u32 {
        self.texture.index()
    }

    // The width of a face at level 0, in texels.
    pub fn extent(&self) -> u32 {
        self.extent
    }

    // The finest level resident, on the CPU as on the GPU.
    pub fn level(&self) -> &HeightMapLevel {
        &self.level
    }

    pub fn bounds(&self) -> &HeightBounds {
        &self.bounds
    }

    // A level's texels, face by face, taken from the front of each face's levels.
    fn level_texels(image: &TextureImage, size: usize) -> Vec<u16> {
        image
            .layers()
            .iter()
            .flat_map(|face| face[..size * size_of::<u16>()].as_chunks::<{ size_of::<u16>() }>().0)
            .map(|&bytes| u16::from_le_bytes(bytes))
            .collect()
    }
}

impl StreamedMips for HeightMap {
    type Options = HeightMapOptions;

    fn load(
        context: &LoadContext,
        path: &Path,
        options: &HeightMapOptions,
        first_level: u32,
        current: Option<&HeightMap>,
    ) -> anyhow::Result<Box<dyn PendingResource<HeightMap>>> {
        let shape = TextureImage::read_levels(path, ColorSpace::Linear, 0..0)?;
        ensure!(
            shape.kind() == TextureKind::CubeMap && shape.format() == TextureFormat::R16Unorm,
            "a height map is an R16_UNORM cube map, not a {:?} {}",
            shape.format(),
            shape.kind()
        );

        let extent = shape.width();
        ensure!(
            extent >= SPLINE_TEXELS as u32,
            "a height map's faces are at least {SPLINE_TEXELS} texels wide, not {extent}"
        );
        let mip_levels = shape.mip_levels().min((extent / SPLINE_TEXELS as u32).ilog2() + 1);
        let first_level = first_level.min(mip_levels - 1);
        let missing_end = current.map_or(mip_levels, |current| current.first_level.max(first_level));
        let image = TextureImage::read_levels(path, ColorSpace::Linear, first_level..missing_end)?;
        let level_extent = (extent >> first_level).max(1);
        let texels = if image.levels().is_empty() {
            let finest = TextureImage::read_levels(path, ColorSpace::Linear, first_level..first_level + 1)?;
            HeightMap::level_texels(&finest, level_extent as usize * level_extent as usize)
        } else {
            HeightMap::level_texels(&image, level_extent as usize * level_extent as usize)
        };
        let level = HeightMapLevel::new(first_level, level_extent, texels, options.minimum, options.maximum)?;
        let bounds = HeightBounds::new(&level);

        Ok(Box::new(PendingHeightMap {
            texture: Texture::upload_faces(context, image, first_level..mip_levels)?,
            heights: Some(ResidentHeights { level, bounds }),
            first_level,
            mip_levels,
            extent,
        }))
    }

    fn image(&self) -> &Image {
        self.texture.image()
    }

    fn first_level(&self) -> u32 {
        self.first_level
    }

    fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    fn memory(&self) -> MemoryUsage {
        let texels = (self.level.extent() * self.level.extent()) as usize * CubeFace::ALL.len();

        MemoryUsage {
            video: self.texture.image().size(),
            ram: (texels * size_of::<u16>() + self.bounds.size()) as u64,
        }
    }
}

// The CPU's part of a height map, moved into it once the upload completes.
struct ResidentHeights {
    level: HeightMapLevel,
    bounds: HeightBounds,
}

struct PendingHeightMap {
    texture: PendingTexture,
    heights: Option<ResidentHeights>,
    first_level: u32,
    mip_levels: u32,
    extent: u32,
}

impl PendingResource<HeightMap> for PendingHeightMap {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<HeightMap>> {
        let Some(texture) = self.texture.poll(bindless)? else {
            return Ok(None);
        };
        let heights = self.heights.take().expect("a pending height map completes once");

        Ok(Some(HeightMap {
            texture,
            first_level: self.first_level,
            mip_levels: self.mip_levels,
            extent: self.extent,
            level: heights.level,
            bounds: heights.bounds,
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
