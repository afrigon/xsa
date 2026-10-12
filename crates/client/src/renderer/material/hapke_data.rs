use super::{HapkeParameters, NO_TEXTURE};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct HapkeData {
    pub scatter_texture: u32,
    pub surge_texture: u32,
    pub porosity: f32,
    pub roughness: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
    pub padding: u32,
}

impl HapkeData {
    pub const NONE: HapkeData = HapkeData {
        scatter_texture: NO_TEXTURE,
        surge_texture: NO_TEXTURE,
        porosity: 0.0,
        roughness: 0.0,
        blend: 0.0,
        light_boost: 0.0,
        gamma_boost: 0.0,
        padding: 0,
    };
}

impl From<HapkeParameters> for HapkeData {
    fn from(hapke: HapkeParameters) -> Self {
        HapkeData {
            scatter_texture: hapke.scatter.index(),
            surge_texture: hapke.surge.index(),
            porosity: hapke.porosity,
            roughness: hapke.roughness,
            blend: hapke.blend,
            light_boost: hapke.light_boost,
            gamma_boost: hapke.gamma_boost,
            padding: 0,
        }
    }
}
