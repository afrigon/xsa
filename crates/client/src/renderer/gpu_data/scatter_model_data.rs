use glam::Vec3;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterModelData {
    // The mesh's bounding box, in the model's frame.
    pub box_center: Vec3,
    // The instance list it adds to, NO_LIST while it cannot be drawn.
    pub list: u32,
    pub box_half_extent: Vec3,
    pub first_instance: u32,
    // Matches `ScatterOrientation` in scatter/cull.slang.
    pub orientation: u32,
    pub padding: [u32; 3],
}

const _: () = assert!(size_of::<ScatterModelData>() == 48);

unsafe impl GpuData for ScatterModelData {}
