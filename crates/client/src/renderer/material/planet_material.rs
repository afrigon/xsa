use glam::Vec3;

use super::hapke_data::HapkeData;
use super::planet_data::PlanetData;
use super::{HapkeParameters, Material, NO_TEXTURE, Shader};
use crate::renderer::TextureHandle;

pub struct PlanetMaterial {
    pub base_color: Vec3,
    pub color: Option<TextureHandle>,
    pub normal: Option<TextureHandle>,
    pub emissive: Option<TextureHandle>,
    pub emissive_luminance: f32,
    pub hapke: Option<HapkeParameters>,
}

impl Material for PlanetMaterial {
    type Data = PlanetData;
    const SHADER: Shader = Shader::Planet;

    fn gpu_data(&self) -> PlanetData {
        let index = |texture: Option<TextureHandle>| texture.map_or(NO_TEXTURE, TextureHandle::index);

        PlanetData {
            base_color: self.base_color,
            color_texture: index(self.color),
            normal_texture: index(self.normal),
            emissive_texture: index(self.emissive),
            emissive_luminance: self.emissive_luminance,
            padding: 0,
            hapke: self.hapke.map_or(HapkeData::NONE, HapkeData::from),
        }
    }
}
