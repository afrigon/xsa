use glam::{Mat3, Mat4};
use resource_vulkan::{CubeMap, ResourceHandle};

use super::skybox_data::SkyboxData;
use super::{Material, Shader};

pub struct SkyboxMaterial {
    pub cube_map: ResourceHandle<CubeMap>,
    pub orientation: Mat3,
    pub luminance: f32,
}

impl Material for SkyboxMaterial {
    type Data = SkyboxData;
    const SHADER: Shader = Shader::Skybox;

    // None until its cube map is loaded.
    fn gpu_data(&self) -> Option<SkyboxData> {
        Some(SkyboxData {
            orientation: Mat4::from_mat3(self.orientation),
            cube_map: self.cube_map.get()?.index(),
            luminance: self.luminance,
            padding: [0; 2],
        })
    }
}
