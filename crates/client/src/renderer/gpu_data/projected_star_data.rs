use glam::{Vec2, Vec3};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ProjectedStarData {
    pub screen_position: Vec2,
    // Fraction of the star's disk no body hides.
    pub visibility: f32,
    pub angular_radius: f32,
    pub illuminance: Vec3,
    pub padding: u32,
}

const _: () = assert!(size_of::<ProjectedStarData>() == 32);

unsafe impl GpuData for ProjectedStarData {}
