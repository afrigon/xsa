use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EmissiveData {
    pub(super) luminance: Vec3,
    pub(super) padding: u32,
}

const _: () = assert!(size_of::<EmissiveData>() == 16);

unsafe impl GpuData for EmissiveData {}

impl ShaderData for EmissiveData {
    const SHADER: Shader = Shader::Emissive;
}
