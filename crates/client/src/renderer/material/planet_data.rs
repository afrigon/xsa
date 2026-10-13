use glam::Vec3;

use super::hapke_data::HapkeData;
use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlanetData {
    pub(super) base_color: Vec3,
    pub(super) color_texture: u32,
    pub(super) normal_texture: u32,
    pub(super) emissive_texture: u32,
    pub(super) emissive_luminance: f32,
    pub(super) padding: u32,
    pub(super) hapke: HapkeData,
}

const _: () = assert!(size_of::<PlanetData>() == 40);

unsafe impl GpuData for PlanetData {}
