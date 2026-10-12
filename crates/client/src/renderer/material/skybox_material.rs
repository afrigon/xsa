use glam::{Mat3, Mat4};

use super::skybox_data::SkyboxData;
use super::{Material, Shader};
use crate::renderer::CubeMapHandle;

pub struct SkyboxMaterial {
    pub cube_map: CubeMapHandle,
    pub orientation: Mat3,
    pub luminance: f32,
}

impl Material for SkyboxMaterial {
    type Data = SkyboxData;
    const SHADER: Shader = Shader::Skybox;

    fn gpu_data(&self) -> SkyboxData {
        SkyboxData {
            orientation: Mat4::from_mat3(self.orientation),
            cube_map: self.cube_map.index(),
            luminance: self.luminance,
            padding: [0; 2],
        }
    }
}
