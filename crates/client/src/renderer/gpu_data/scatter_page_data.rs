use ash::vk;
use glam::Vec3;

use super::GpuData;

// A page's tile placed as `TileData` places one: centers relative to each frame's camera.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterPageData {
    pub center: Vec3,
    pub instance_count: u32,
    pub previous_center: Vec3,
    // Its scatter use in the frame's `ScatterUseData`.
    pub scatter_use: u32,
    pub cube_center: Vec3,
    pub half_size: f32,
    pub column_axis: Vec3,
    pub x: u32,
    pub row_axis: Vec3,
    pub y: u32,
    pub instances: vk::DeviceAddress,
    pub depth: u32,
    pub face: u32,
}

const _: () = assert!(size_of::<ScatterPageData>() == 96);

unsafe impl GpuData for ScatterPageData {}
