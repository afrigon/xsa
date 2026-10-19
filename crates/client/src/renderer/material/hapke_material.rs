use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::hapke_data::HapkeData;
use super::{Material, MaterialData, texture_index};

// A terrain shaded with Hapke 2012. Its albedo, normals and Hapke parameters are terrain layers the tiles read; where
// a tile has no Hapke parameters, it is shaded with Lambert.
pub struct HapkeMaterial {
    pub base_color: Vec3,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
}

impl Material for HapkeMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(HapkeData {
            base_color: self.base_color,
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            padding: [0; 3],
        }))
    }
}
