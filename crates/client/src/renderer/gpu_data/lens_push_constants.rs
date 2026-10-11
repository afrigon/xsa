use ash::vk;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct LensPushConstants {
    pub frame: vk::DeviceAddress,
    pub lights: vk::DeviceAddress,
    pub ghosts: vk::DeviceAddress,
    pub glare_intensity: f32,
    pub glare_falloff: f32,
    pub focal_length_pixels: f32,
    pub light_count: u32,
    pub starburst_texture: u32,
    pub starburst_inner: f32,
    pub starburst_extent: f32,
    pub ghost_count: u32,
}

const _: () = assert!(size_of::<LensPushConstants>() == 56);

unsafe impl GpuData for LensPushConstants {}
