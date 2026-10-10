use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct StarVisibilityPushConstants {
    pub frame: vk::DeviceAddress,
    pub stars: vk::DeviceAddress,
    pub visibility: vk::DeviceAddress,
    pub depth_texture: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<StarVisibilityPushConstants>() == 32);

unsafe impl GpuData for StarVisibilityPushConstants {}
