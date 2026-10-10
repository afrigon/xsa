use ash::vk;
use glam::{Mat4, Vec2};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct FrameData {
    pub view_projection: Mat4,
    pub world_from_clip: Mat4,
    pub previous_view_projection: Mat4,
    pub stars: vk::DeviceAddress,
    pub star_count: u32,
    pub padding: u32,
    pub viewport_size: Vec2,
    pub jitter: Vec2,
    pub exposure: f32,
    pub tonemapper: u32,
    pub starlight_illuminance: f32,
    pub shading_model: u32,
}

const _: () = assert!(size_of::<FrameData>() == 240);

unsafe impl GpuData for FrameData {}
