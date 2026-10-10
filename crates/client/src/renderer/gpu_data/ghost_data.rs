use glam::Vec3;

use super::GpuData;

// One lens ghost; the fields after `shape` are read by that shape only.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct GhostData {
    pub position: f32,
    pub size: f32,
    pub aspect_ratio: f32,
    pub intensity: f32,
    pub tint: Vec3,
    pub shape: u32,
    pub radius: f32,
    pub width: f32,
    pub dispersion: f32,
    pub softness: f32,
    pub streaks: u32,
    pub spots: f32,
    pub texture: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<GhostData>() == 64);

unsafe impl GpuData for GhostData {}
