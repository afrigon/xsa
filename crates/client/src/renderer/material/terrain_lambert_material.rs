use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::terrain_lambert_data::TerrainLambertData;
use super::{Material, MaterialData, texture_index};

// A terrain shaded with Lambert. Its albedo and normals are terrain layers the tiles read; without them, the base color
// on the sphere.
pub struct TerrainLambertMaterial {
    pub base_color: Vec3,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
}

impl Material for TerrainLambertMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(TerrainLambertData {
            base_color: self.base_color,
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            padding: [0; 3],
        }))
    }
}
