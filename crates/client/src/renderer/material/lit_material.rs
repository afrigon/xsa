use glam::Vec3;

use super::lit_data::LitData;
use super::{Material, Shader};

pub struct LitMaterial {
    pub base_color: Vec3,
}

impl Material for LitMaterial {
    type Data = LitData;
    const SHADER: Shader = Shader::Lit;

    fn gpu_data(&self) -> LitData {
        LitData {
            base_color: self.base_color,
            padding: 0,
        }
    }
}
