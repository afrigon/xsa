use glam::{Mat4, Vec2, Vec4};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct FrameData {
    pub view_projection: Mat4,
    pub world_from_clip: Mat4,
    pub previous_view_projection: Mat4,
    pub sun_position: Vec4,
    pub sun_intensity: Vec4,
    pub viewport_size: Vec2,
    pub jitter: Vec2,
    pub exposure: f32,
    pub scene_color_texture: u32,
    pub tonemapper: u32,
    pub starlight_illuminance: f32,
    pub bloom_texture: u32,
    pub bloom_strength: f32,
    pub shading_model: u32,
    pub padding: [u32; 1],
}

const _: () = assert!(size_of::<FrameData>() == 272);

unsafe impl GpuData for FrameData {}
