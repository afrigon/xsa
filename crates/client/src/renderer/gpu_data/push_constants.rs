use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct PushConstants {
    pub frame: vk::DeviceAddress,
    pub material: vk::DeviceAddress,
    // The draw's record, laid out by its geometry family; 0 for a family without one.
    pub draw: vk::DeviceAddress,
}

const _: () = assert!(size_of::<PushConstants>() == 24);

unsafe impl GpuData for PushConstants {}
