use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterPushConstants {
    pub frame: vk::DeviceAddress,
    pub objects: vk::DeviceAddress,
    pub terrains: vk::DeviceAddress,
    pub pages: vk::DeviceAddress,
    pub uses: vk::DeviceAddress,
    pub models: vk::DeviceAddress,
    // One counter per instance list, then the count of instances within range.
    pub counts: vk::DeviceAddress,
    pub instances: vk::DeviceAddress,
    pub list_count: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<ScatterPushConstants>() == 72);

unsafe impl GpuData for ScatterPushConstants {}
