use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramPushConstants {
    pub frame: vk::DeviceAddress,
    pub histogram: vk::DeviceAddress,
    pub albedo_histogram: vk::DeviceAddress,
    pub scene_color_texture: u32,
    pub lens_texture: u32,
    pub illuminance_texture: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<HistogramPushConstants>() == 40);

unsafe impl GpuData for HistogramPushConstants {}
