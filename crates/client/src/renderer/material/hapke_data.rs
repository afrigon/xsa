use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HapkeData {
    pub(super) base_color: Vec3,
    pub(super) lobe_width: f32,
    pub(super) lobe_balance: f32,
    // Radians.
    pub(super) macroscopic_roughness: f32,
    pub(super) porosity: f32,
    pub(super) shadow_hiding_amplitude: f32,
    pub(super) shadow_hiding_width: f32,
    pub(super) backscatter_amplitude: f32,
    pub(super) backscatter_width: f32,
    pub(super) padding: u32,
}

const _: () = assert!(size_of::<HapkeData>() == 48);

unsafe impl GpuData for HapkeData {}

impl ShaderData for HapkeData {
    const SHADER: Shader = Shader::Hapke;
}
