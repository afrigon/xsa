use glam::Vec3;

use super::GpuData;

// One lens ghost: a texture centered on the line from the light through the screen center.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct GhostData {
    pub position: f32,
    // Angular diameter, in radians.
    pub size: f32,
    pub texture: u32,
    // Red moves outward along the axis and blue inward, each by this fraction of the ghost's size per radian the light
    // sits off the view axis.
    pub dispersion_per_radian: f32,
    pub tint: Vec3,
    // Multiplies the texture.
    pub intensity: f32,
}

const _: () = assert!(size_of::<GhostData>() == 32);

unsafe impl GpuData for GhostData {}
