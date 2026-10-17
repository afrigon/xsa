use glam::Vec3;

use super::hapke_data::HapkeData;
use super::planet_data::PlanetData;
use super::{HapkeParameters, Material, NO_TEXTURE, Shader};
use resource_vulkan::{ResourceHandle, Texture};

pub struct PlanetMaterial {
    pub base_color: Vec3,
    pub color: Option<ResourceHandle<Texture>>,
    pub normal: Option<ResourceHandle<Texture>>,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
    pub hapke: Option<HapkeParameters>,
}

impl Material for PlanetMaterial {
    type Data = PlanetData;
    const SHADER: Shader = Shader::Planet;

    fn gpu_data(&self) -> Option<PlanetData> {
        let index = |texture: &Option<ResourceHandle<Texture>>| {
            texture
                .as_ref()
                .and_then(ResourceHandle::get)
                .map_or(NO_TEXTURE, Texture::index)
        };

        Some(PlanetData {
            base_color: self.base_color,
            color_texture: index(&self.color),
            normal_texture: index(&self.normal),
            emissive_texture: index(&self.emissive),
            emissive_luminance: self.emissive_luminance,
            padding: 0,
            hapke: HapkeData::of(self.hapke.as_ref()),
        })
    }
}
