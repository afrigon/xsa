use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::lambert_data::LambertData;
use super::{Material, MaterialData, texture_index};

pub struct LambertMaterial {
    pub base_color: Vec3,
    pub color: Option<ResourceHandle<Texture>>,
    pub normal: Option<ResourceHandle<Texture>>,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
}

impl Material for LambertMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(LambertData {
            base_color: self.base_color,
            color_texture: texture_index(self.color.as_ref()),
            normal_texture: texture_index(self.normal.as_ref()),
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            padding: 0,
        }))
    }
}
