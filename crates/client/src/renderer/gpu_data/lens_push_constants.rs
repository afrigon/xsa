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
    pub spikes_intensity: f32,
    pub spikes_length: f32,
    pub spikes_width: f32,
    pub spikes_dispersion: f32,
    pub spikes_texture: u32,
    pub blades: u32,
    pub rotation: f32,
    pub ghost_count: u32,
}

const _: () = assert!(size_of::<LensPushConstants>() == 72);

unsafe impl GpuData for LensPushConstants {}
