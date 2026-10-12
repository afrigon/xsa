use glam::Vec3;

use crate::renderer::gpu_data::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EmissiveData {
    pub(super) luminance: Vec3,
    pub(super) padding: u32,
}

const _: () = assert!(size_of::<EmissiveData>() == 16);

unsafe impl GpuData for EmissiveData {}
