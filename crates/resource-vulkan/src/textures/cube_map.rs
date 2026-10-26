use std::path::Path;

use memory_vulkan::{BindlessTable, Image, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureImage};

use crate::textures::{PendingTexture, Texture, TextureUpload};
use crate::{LoadContext, MemoryUsage, PendingResource, ResourceKind, StreamedMips};

/// A cube map in GPU memory, sampled through its slot in the cube map binding. Loaded as a `ResourceKind`, it holds
/// its file's whole chain; streamed as `StreamedMips`, the levels from `first_level` down.
pub struct CubeMap {
    texture: Texture,
    first_level: u32,
    mip_levels: u32,
}

impl CubeMap {
    /// Where shaders find it in the cube map binding's array.
    pub fn index(&self) -> u32 {
        self.texture.index()
    }
}

impl ResourceKind for CubeMap {
    type Options = ColorSpace;

    fn load(
        context: &LoadContext,
        path: &Path,
        color_space: &ColorSpace,
    ) -> anyhow::Result<Box<dyn PendingResource<CubeMap>>> {
        let image = TextureImage::read(path, *color_space)?;
        let mip_levels = image.mip_levels();

        Ok(Box::new(PendingCubeMap {
            texture: TextureUpload::new(image).cube_map(context)?,
            first_level: 0,
            mip_levels,
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
    }
}

impl StreamedMips for CubeMap {
    type Options = ColorSpace;

    fn load(
        context: &LoadContext,
        path: &Path,
        color_space: &ColorSpace,
        first_level: u32,
        current: Option<&CubeMap>,
    ) -> anyhow::Result<Box<dyn PendingResource<CubeMap>>> {
        let mip_levels = TextureImage::read_levels(path, *color_space, 0..0)?.mip_levels();
        let first_level = first_level.min(mip_levels - 1);
        let missing_end = current.map_or(mip_levels, |current| current.first_level.max(first_level));
        let image = TextureImage::read_levels(path, *color_space, first_level..missing_end)?;

        Ok(Box::new(PendingCubeMap {
            texture: TextureUpload::new(image).cube_map_levels(context, first_level..mip_levels)?,
            first_level,
            mip_levels,
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
        self.texture.memory()
    }
}

struct PendingCubeMap {
    texture: PendingTexture,
    first_level: u32,
    mip_levels: u32,
}

impl PendingResource<CubeMap> for PendingCubeMap {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<CubeMap>> {
        Ok(self.texture.poll(bindless)?.map(|texture| CubeMap {
            texture,
            first_level: self.first_level,
            mip_levels: self.mip_levels,
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
