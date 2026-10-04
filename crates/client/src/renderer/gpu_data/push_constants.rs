use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct PushConstants {
    pub frame: vk::DeviceAddress,
    pub objects: vk::DeviceAddress,
    pub materials: vk::DeviceAddress,
    pub object_index: u32,
    pub material_index: u32,
}

const _: () = assert!(size_of::<PushConstants>() == 32);

unsafe impl GpuData for PushConstants {}
