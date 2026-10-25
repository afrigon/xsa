use glam::Vec3;

use super::emissive_data::EmissiveData;
use super::{Material, MaterialData};

pub struct EmissiveMaterial {
    pub luminance: Vec3,
}

impl Material for EmissiveMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(EmissiveData {
            luminance: self.luminance,
            padding: 0,
        }))
    }
}
