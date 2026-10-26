use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerrainLambertData {
    pub(super) base_color: Vec3,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) padding: [u32; 3],
}

const _: () = assert!(size_of::<TerrainLambertData>() == 32);

unsafe impl GpuData for TerrainLambertData {}

impl ShaderData for TerrainLambertData {
    const SHADER: Shader = Shader::TerrainLambert;
}
