use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerrainEmissiveData {
    pub(super) luminance: Vec3,
    pub(super) padding: u32,
}

const _: () = assert!(size_of::<TerrainEmissiveData>() == 16);

unsafe impl GpuData for TerrainEmissiveData {}

impl ShaderData for TerrainEmissiveData {
    const SHADER: Shader = Shader::TerrainEmissive;
}
