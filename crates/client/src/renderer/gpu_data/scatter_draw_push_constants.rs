use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterDrawPushConstants {
    pub draws: vk::DeviceAddress,
    pub counts: vk::DeviceAddress,
    pub arguments: vk::DeviceAddress,
    pub draw_count: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<ScatterDrawPushConstants>() == 32);

unsafe impl GpuData for ScatterDrawPushConstants {}
