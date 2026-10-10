use glam::{Vec2, Vec3};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ProjectedStarData {
    pub screen_position: Vec2,
    pub pixel_radius: f32,
    // Depth-buffer value above which a sample hides the star.
    pub occlusion_depth: f32,
    pub illuminance: Vec3,
    pub angular_radius: f32,
}

const _: () = assert!(size_of::<ProjectedStarData>() == 32);

unsafe impl GpuData for ProjectedStarData {}
