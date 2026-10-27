use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TerrainDrawData {
    pub objects: vk::DeviceAddress,
    pub tiles: vk::DeviceAddress,
    // The maps of the terrain drawn.
    pub terrain: vk::DeviceAddress,
    // Where the camera stands in the terrain's surfaces; 0 for a terrain without surfaces.
    pub surface_anchors: vk::DeviceAddress,
    pub object_index: u32,
    pub first_tile: u32,
}

const _: () = assert!(size_of::<TerrainDrawData>() == 40);

unsafe impl GpuData for TerrainDrawData {}
