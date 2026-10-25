use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct PushConstants {
    pub frame: vk::DeviceAddress,
    pub objects: vk::DeviceAddress,
    pub material: vk::DeviceAddress,
    pub object_index: u32,
    pub first_tile: u32,
    pub tiles: vk::DeviceAddress,
    // The pages each tile reads, by tile index.
    pub tile_pages: vk::DeviceAddress,
    // Where the camera stands in the terrain's surfaces; 0 for a terrain without surfaces.
    pub surface_anchors: vk::DeviceAddress,
}

const _: () = assert!(size_of::<PushConstants>() == 56);

unsafe impl GpuData for PushConstants {}
