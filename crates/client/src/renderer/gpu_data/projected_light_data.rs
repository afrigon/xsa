use glam::{Vec2, Vec3};

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ProjectedLightData {
    pub screen_position: Vec2,
    // Fraction of the light source's disk no body hides.
    pub visibility: f32,
    pub angular_radius: f32,
    pub illuminance: Vec3,
    // Fraction of the light the lens passes at this angle off its axis.
    pub acceptance: f32,
}

const _: () = assert!(size_of::<ProjectedLightData>() == 32);

unsafe impl GpuData for ProjectedLightData {}
