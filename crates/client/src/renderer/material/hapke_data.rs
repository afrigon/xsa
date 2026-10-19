use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

// The terrain layers it shades with are each tile's pages, not material parameters.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HapkeData {
    pub(super) base_color: Vec3,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) padding: [u32; 3],
}

const _: () = assert!(size_of::<HapkeData>() == 32);

unsafe impl GpuData for HapkeData {}

impl ShaderData for HapkeData {
    const SHADER: Shader = Shader::Hapke;
}
