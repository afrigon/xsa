use std::path::Path;

use glam::Vec3;
use memory_vulkan::{BindlessTable, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureImage};
use resource_vulkan::{LoadContext, MemoryUsage, PendingResource, PendingTexture, ResourceKind, Texture};

// The lens's starburst, a log-polar texture, and the brightest value at each row's angle or beyond, which bounds
// how far a light's starburst reaches.
pub struct LensStarburst {
    texture: Texture,
    maxima: Vec<f32>,
}

impl LensStarburst {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn maxima(&self) -> &[f32] {
        &self.maxima
    }

    fn maxima_of(starburst: &TextureImage) -> anyhow::Result<Vec<f32>> {
        let texels = starburst.rgb_texels()?;
        let mut maxima: Vec<f32> = texels
            .chunks_exact(starburst.width() as usize)
            .map(|row| {
                row.iter()
                    .fold(0.0_f32, |peak, &texel| peak.max(Vec3::from(texel).max_element()))
            })
            .collect();

        for row in (0..maxima.len().saturating_sub(1)).rev() {
            maxima[row] = maxima[row].max(maxima[row + 1]);
        }

        Ok(maxima)
    }
}

impl ResourceKind for LensStarburst {
    type Options = ();

    fn load(context: &LoadContext, path: &Path, _: &()) -> anyhow::Result<Box<dyn PendingResource<LensStarburst>>> {
        let image = TextureImage::read(path, ColorSpace::Linear)?;
        let maxima = LensStarburst::maxima_of(&image)?;

        Ok(Box::new(PendingStarburst {
            texture: Texture::upload(context, image)?,
            maxima,
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
            + MemoryUsage {
                video: 0,
                ram: size_of_val(self.maxima.as_slice()) as u64,
            }
    }
}

struct PendingStarburst {
    texture: PendingTexture,
    maxima: Vec<f32>,
}

impl PendingResource<LensStarburst> for PendingStarburst {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<LensStarburst>> {
        Ok(self.texture.poll(bindless)?.map(|texture| LensStarburst {
            texture,
            maxima: std::mem::take(&mut self.maxima),
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
