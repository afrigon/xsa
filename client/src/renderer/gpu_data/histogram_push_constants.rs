use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramPushConstants {
    pub frame: vk::DeviceAddress,
    pub histogram: vk::DeviceAddress,
}

unsafe impl GpuData for HistogramPushConstants {}
