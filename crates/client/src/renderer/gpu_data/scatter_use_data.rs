use glam::Vec3;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct ScatterUseData {
    pub object_index: u32,
    pub terrain_index: u32,
    // Its first model in the frame's `ScatterModelData`.
    pub first_model: u32,
    // Meters.
    pub range: f32,
    pub tint: Vec3,
    // Meters: the radius of the body's lowest ground.
    pub lowest_radius: f32,
    // From the body's center to the camera, in radii of its lowest ground.
    pub camera_from_center: Vec3,
    // The square of that distance less one; at or below 0 inside the lowest ground.
    pub horizon_squared: f32,
    // Linear: the ground's average albedo, until its colour layer loads.
    pub ground_albedo: Vec3,
    pub padding: u32,
}

const _: () = assert!(size_of::<ScatterUseData>() == 64);

unsafe impl GpuData for ScatterUseData {}
