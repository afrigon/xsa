use std::path::Path;

use memory_vulkan::{BindlessTable, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureImage};

use crate::textures::{PendingTexture, Texture, TextureUpload};
use crate::{LoadContext, MemoryUsage, PendingResource, ResourceKind};

/// A cube map in GPU memory, sampled through its slot in the cube map binding.
pub struct CubeMap {
    texture: Texture,
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

        Ok(Box::new(PendingCubeMap {
            texture: TextureUpload::new(image).cube_map(context)?,
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
    }
}

struct PendingCubeMap {
    texture: PendingTexture,
}

impl PendingResource<CubeMap> for PendingCubeMap {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<CubeMap>> {
        Ok(self.texture.poll(bindless)?.map(|texture| CubeMap { texture }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
