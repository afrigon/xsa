use std::path::Path;

use ash::vk;
use glam::Vec3;
use xsa_packs::{GhostShape, LensDefinition, SpikesShape};

use super::TextureHandle;
use super::gpu_data::GhostData;
use super::material::NO_TEXTURE;
use crate::vulkan::{Allocator, Buffer, Device, MemoryLocation};

// Must match the ghost shapes in common/lens.slang.
const GHOST_SHAPE_RING: u32 = 0;
const GHOST_SHAPE_POLYGON: u32 = 1;
const GHOST_SHAPE_TEXTURE: u32 = 2;

pub(in crate::renderer) struct Lens {
    pub glare_intensity: f32,
    pub glare_falloff: f32,
    pub spikes_intensity: f32,
    pub spikes_length: f32,
    pub spikes_width: f32,
    pub spikes_dispersion: f32,
    pub spikes_texture: u32,
    pub blades: u32,
    pub rotation: f32,
    pub ghost_count: u32,
    ghost_data: Vec<GhostData>,
    ghosts: Option<Buffer>,
}

impl Lens {
    pub fn new(
        definition: &LensDefinition,
        mut load_texture: impl FnMut(&str, &Path) -> anyhow::Result<TextureHandle>,
    ) -> anyhow::Result<Lens> {
        let (spikes_width, spikes_dispersion, spikes_texture) = match &definition.spikes.shape {
            SpikesShape::Computed { width, dispersion } => (*width, *dispersion, NO_TEXTURE),
            SpikesShape::Texture(path) => (0.0, 0.0, load_texture("lens spikes", path)?.index()),
        };
        let ghost_data = definition
            .ghosts
            .iter()
            .enumerate()
            .map(|(index, ghost)| {
                let mut data = GhostData {
                    position: ghost.position,
                    size: ghost.size,
                    aspect_ratio: ghost.aspect_ratio,
                    intensity: ghost.intensity,
                    tint: Vec3::ONE,
                    shape: GHOST_SHAPE_RING,
                    radius: 0.0,
                    width: 0.0,
                    dispersion: 0.0,
                    softness: 0.0,
                    streaks: 0,
                    spots: 0.0,
                    texture: NO_TEXTURE,
                    padding: 0,
                };

                match &ghost.shape {
                    GhostShape::Ring(ring) => {
                        data.radius = ring.radius;
                        data.width = ring.width;
                        data.dispersion = ring.dispersion;
                        data.streaks = ring.streaks;
                        data.spots = ring.spots;
                    }
                    GhostShape::Polygon(polygon) => {
                        data.shape = GHOST_SHAPE_POLYGON;
                        data.softness = polygon.softness;
                        data.tint = polygon.tint;
                    }
                    GhostShape::Texture(path) => {
                        data.shape = GHOST_SHAPE_TEXTURE;
                        data.texture = load_texture(&format!("lens ghost {index}"), path)?.index();
                    }
                }

                Ok(data)
            })
            .collect::<anyhow::Result<Vec<GhostData>>>()?;

        Ok(Lens {
            glare_intensity: definition.glare.intensity,
            glare_falloff: definition.glare.falloff,
            spikes_intensity: definition.spikes.intensity,
            spikes_length: definition.spikes.length_degrees.to_radians(),
            spikes_width,
            spikes_dispersion,
            spikes_texture,
            blades: definition.aperture.blades,
            rotation: definition.aperture.rotation_degrees.to_radians(),
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
