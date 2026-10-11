use std::path::Path;

use ash::vk;
use xsa_packs::LensDefinition;

use super::TextureHandle;
use super::gpu_data::GhostData;
use crate::vulkan::{Allocator, Buffer, Device, MemoryLocation};

pub(in crate::renderer) struct Lens {
    pub glare_intensity: f32,
    pub glare_falloff: f32,
    pub starburst_texture: u32,
    pub starburst_inner: f32,
    pub starburst_extent: f32,
    pub ghost_count: u32,
    ghost_data: Vec<GhostData>,
    ghosts: Option<Buffer>,
}

impl Lens {
    pub fn new(
        definition: &LensDefinition,
        mut load_texture: impl FnMut(&str, &Path) -> anyhow::Result<TextureHandle>,
    ) -> anyhow::Result<Lens> {
        let starburst_texture = load_texture("lens starburst", &definition.starburst.texture)?.index();
        let ghost_data = definition
            .ghosts
            .iter()
            .enumerate()
            .map(|(index, ghost)| {
                Ok(GhostData {
                    position: ghost.position,
                    size: ghost.size_degrees.to_radians(),
                    texture: load_texture(&format!("lens ghost {index}"), &ghost.texture)?.index(),
                    padding: 0,
                })
            })
            .collect::<anyhow::Result<Vec<GhostData>>>()?;

        Ok(Lens {
            glare_intensity: definition.glare.intensity,
            glare_falloff: definition.glare.falloff,
            starburst_texture,
            starburst_inner: definition.starburst.inner_degrees.to_radians(),
            starburst_extent: definition.starburst.extent_degrees.to_radians(),
            ghost_count: ghost_data.len() as u32,
            ghost_data,
            ghosts: None,
        })
    }

    pub fn upload(&mut self, device: &Device, allocator: &mut Allocator) -> anyhow::Result<()> {
        if self.ghost_data.is_empty() {
            return Ok(());
        }

        let mut ghosts = Buffer::new(
            device,
            allocator,
            "lens ghosts",
            size_of_val(self.ghost_data.as_slice()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
        )?;

        if let Err(err) = ghosts.write(&self.ghost_data) {
            unsafe { ghosts.destroy(device, allocator) };
            return Err(err);
        }

        self.ghosts = Some(ghosts);

        Ok(())
    }

    pub fn ghosts(&self) -> vk::DeviceAddress {
        self.ghosts.as_ref().map_or(0, Buffer::device_address)
    }

    // Safety: only once the GPU has finished every frame that used the lens.
    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        if let Some(ghosts) = &mut self.ghosts {
            unsafe { ghosts.destroy(device, allocator) };
        }
    }
}
