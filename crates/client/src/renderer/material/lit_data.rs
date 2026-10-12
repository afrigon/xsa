use glam::Vec3;

use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LitData {
    pub(super) base_color: Vec3,
    pub(super) padding: u32,
}

const _: () = assert!(size_of::<LitData>() == 16);

unsafe impl GpuData for LitData {}
