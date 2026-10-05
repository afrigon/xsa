use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TaaPushConstants {
    pub frame: vk::DeviceAddress,
    pub color_texture: u32,
    pub motion_texture: u32,
    pub depth_texture: u32,
    pub history_texture: u32,
    pub output_image: u32,
    pub history_valid: u32,
    pub history_exposure_scale: f32,
    pub padding: u32,
}

const _: () = assert!(size_of::<TaaPushConstants>() == 40);

unsafe impl GpuData for TaaPushConstants {}
