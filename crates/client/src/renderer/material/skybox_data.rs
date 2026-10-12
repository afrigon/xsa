use glam::Mat4;

use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkyboxData {
    pub(super) orientation: Mat4,
    pub(super) cube_map: u32,
    pub(super) luminance: f32,
    pub(super) padding: [u32; 2],
}

const _: () = assert!(size_of::<SkyboxData>() == 80);

unsafe impl GpuData for SkyboxData {}
