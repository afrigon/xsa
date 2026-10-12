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
}

const _: () = assert!(size_of::<PushConstants>() == 40);

unsafe impl GpuData for PushConstants {}
