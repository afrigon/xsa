use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LambertData {
    pub(super) base_color: Vec3,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) padding: [u32; 3],
}

const _: () = assert!(size_of::<LambertData>() == 32);

unsafe impl GpuData for LambertData {}

impl ShaderData for LambertData {
    const SHADER: Shader = Shader::Lambert;
}
