use glam::Vec4;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct StarData {
    pub position_and_radius: Vec4,
    pub intensity: Vec4,
}

const _: () = assert!(size_of::<StarData>() == 32);

unsafe impl GpuData for StarData {}
