use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HapkeData {
    pub(super) base_color: Vec3,
    pub(super) color_texture: u32,
    pub(super) normal_texture: u32,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) medium_texture: u32,
    pub(super) opposition_texture: u32,
    pub(super) padding: [u32; 3],
}

const _: () = assert!(size_of::<HapkeData>() == 48);

unsafe impl GpuData for HapkeData {}

impl ShaderData for HapkeData {
    const SHADER: Shader = Shader::Hapke;
}
