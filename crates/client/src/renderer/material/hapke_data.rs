use glam::Vec3;

use super::Shader;
use super::shader_data::ShaderData;
use crate::renderer::gpu_data::{BiomeData, GpuData, SurfaceData};
use crate::renderer::surface::SURFACE_CLASSES;

// The terrain layers it shades with are each tile's pages, not material parameters. Its surfaces are read only once
// every class's arrays are loaded.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HapkeData {
    pub(super) base_color: Vec3,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) surfaces_loaded: u32,
    pub(super) padding: [u32; 2],
    pub(super) biome: BiomeData,
    pub(super) surfaces: [SurfaceData; SURFACE_CLASSES],
}

const _: () = assert!(size_of::<HapkeData>() == 256);

unsafe impl GpuData for HapkeData {}

impl ShaderData for HapkeData {
    const SHADER: Shader = Shader::Hapke;
}
