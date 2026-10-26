use ash::vk;
use glam::{Mat4, Vec2, Vec4};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct FrameData {
    pub view_projection: Mat4,
    pub world_from_clip: Mat4,
    pub previous_view_projection: Mat4,
    // Without the antialiasing jitter: culling keeps the same frustum from frame to frame.
    pub unjittered_view_projection: Mat4,
    pub stars: vk::DeviceAddress,
    pub star_count: u32,
    pub starfield_exposure: f32,
    pub viewport_size: Vec2,
    pub jitter: Vec2,
    pub exposure: f32,
    pub tonemapper: u32,
    pub starlight_illuminance: f32,
    // The angle one vertex spacing of terrain covers on screen.
    pub vertex_spacing_angle: f32,
    // The angle one pixel covers at the screen's center.
    pub pixel_angle: f32,
    // Multiplies the terrain's heights as they are drawn.
    pub terrain_height_scale: f32,
    // 1 for meshes to dissolve as they fade, 0 for them to pop where their fades reach half.
    pub fade_enabled: u32,
    pub padding: u32,
    // Rows of the linear sRGB white balance matrix, applied before tone mapping.
    pub white_balance: [Vec4; 3],
}

const _: () = assert!(size_of::<FrameData>() == 368);

unsafe impl GpuData for FrameData {}
