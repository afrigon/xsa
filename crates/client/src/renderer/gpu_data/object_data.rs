use glam::Mat4;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ObjectData {
    pub world_from_model: Mat4,
    pub previous_clip_from_model: Mat4,
}

const _: () = assert!(size_of::<ObjectData>() == 128);

unsafe impl GpuData for ObjectData {}
