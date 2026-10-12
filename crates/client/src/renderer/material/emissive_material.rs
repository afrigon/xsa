use glam::Vec3;

use super::emissive_data::EmissiveData;
use super::{Material, Shader};

pub struct EmissiveMaterial {
    pub luminance: Vec3,
}

impl Material for EmissiveMaterial {
    type Data = EmissiveData;
    const SHADER: Shader = Shader::Emissive;

    fn gpu_data(&self) -> EmissiveData {
        EmissiveData {
            luminance: self.luminance,
            padding: 0,
        }
    }
}
