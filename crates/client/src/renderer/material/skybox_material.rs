use glam::{Mat3, Mat4};
use resource_vulkan::{CubeMap, ResourceHandle};

use super::skybox_data::SkyboxData;
use super::{Material, MaterialData};

pub struct SkyboxMaterial {
    pub cube_map: ResourceHandle<CubeMap>,
    pub orientation: Mat3,
    pub luminance: f32,
}

impl Material for SkyboxMaterial {
    // None until its cube map is loaded.
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(SkyboxData {
            orientation: Mat4::from_mat3(self.orientation),
            cube_map: self.cube_map.get()?.index(),
            luminance: self.luminance,
            padding: [0; 2],
        }))
    }
}
