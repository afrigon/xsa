use std::path::Path;

use glam::Vec3;
use memory_vulkan::{BindlessTable, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureImage};
use resource_vulkan::{LoadContext, MemoryUsage, PendingResource, PendingTexture, ResourceKind, Texture};

// A lens ghost's texture and its brightest texel, which bounds the light a ghost adds.
pub struct LensGhost {
    texture: Texture,
    peak: f32,
}

impl LensGhost {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn peak(&self) -> f32 {
        self.peak
    }
}

impl ResourceKind for LensGhost {
    type Options = ();

    fn load(context: &LoadContext, path: &Path, _: &()) -> anyhow::Result<Box<dyn PendingResource<LensGhost>>> {
        let image = TextureImage::read(path, ColorSpace::Linear)?;
        let peak = image
            .rgb_texels()?
            .iter()
            .fold(0.0_f32, |peak, &texel| peak.max(Vec3::from(texel).max_element()));

        Ok(Box::new(PendingGhost {
            texture: Texture::upload(context, image)?,
            peak,
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
    }
}

struct PendingGhost {
    texture: PendingTexture,
    peak: f32,
}

impl PendingResource<LensGhost> for PendingGhost {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<LensGhost>> {
        Ok(self.texture.poll(bindless)?.map(|texture| LensGhost {
            texture,
            peak: self.peak,
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
