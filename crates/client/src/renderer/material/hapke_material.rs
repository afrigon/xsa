use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::hapke_data::HapkeData;
use super::lambert_data::LambertData;
use super::{Material, MaterialData, texture_index};

pub struct HapkeMaterial {
    pub base_color: Vec3,
    pub color: Option<ResourceHandle<Texture>>,
    pub normal: Option<ResourceHandle<Texture>>,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
    // Hapke 2012's parameters other than the albedo, which comes from the color map, stored as published: the
    // particulate medium (b, c, θ̄ in degrees, porosity K) and the opposition effects (B_S0, h_S, B_C0, h_C).
    pub medium: ResourceHandle<Texture>,
    pub opposition: ResourceHandle<Texture>,
}

impl Material for HapkeMaterial {
    // Lambert until both Hapke maps are ready.
    fn gpu_data(&self) -> Option<MaterialData> {
        let (Some(medium), Some(opposition)) = (self.medium.get(), self.opposition.get()) else {
            return Some(MaterialData::new(LambertData {
                base_color: self.base_color,
                color_texture: texture_index(self.color.as_ref()),
                normal_texture: texture_index(self.normal.as_ref()),
                emissive_texture: texture_index(self.emissive.as_ref()),
                emissive_luminance: self.emissive_luminance,
                padding: 0,
            }));
        };

        Some(MaterialData::new(HapkeData {
            base_color: self.base_color,
            color_texture: texture_index(self.color.as_ref()),
            normal_texture: texture_index(self.normal.as_ref()),
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            medium_texture: medium.index(),
            opposition_texture: opposition.index(),
            padding: [0; 3],
        }))
    }
}
