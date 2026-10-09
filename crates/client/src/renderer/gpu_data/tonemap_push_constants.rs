use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TonemapPushConstants {
    pub frame: vk::DeviceAddress,
    pub scene_color_texture: u32,
    pub bloom_texture: u32,
    pub bloom_strength: f32,
    pub padding: u32,
}

const _: () = assert!(size_of::<TonemapPushConstants>() == 24);

unsafe impl GpuData for TonemapPushConstants {}
