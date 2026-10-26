use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::lambert_data::LambertData;
use super::{Material, MaterialData, texture_index};

// A terrain shaded with Lambert. Its albedo and normals are terrain layers the tiles read; without them, the base color
// on the sphere.
pub struct LambertMaterial {
    pub base_color: Vec3,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
}

impl Material for LambertMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(LambertData {
            base_color: self.base_color,
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            padding: [0; 3],
        }))
    }
}
