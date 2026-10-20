use glam::{Vec2, Vec3};

use super::GpuData;

// The most doublings a surface's scale ladder holds: a largest magnification of up to 2^5.
pub(in crate::renderer) const MAX_SURFACE_SCALES: usize = 6;

// One class's surface as the terrain programs read it. Mirrored by `SurfaceData` in terrain/surface/surface-data.slang.
// Sizes are in meters; each scale's rotation is stored as its cosine and sine. Displacements are compared across the
// two blended materials, the higher showing through by `height_contrast` of the 0 to 1 displacement range.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::renderer) struct SurfaceData {
    pub color_texture: u32,
    pub normal_texture: u32,
    pub displacement_texture: u32,
    pub variant_count: u32,
    pub size: f32,
    pub texels: f32,
    pub displacement_low: f32,
    pub displacement_high: f32,
    pub average_color: Vec3,
    pub scale_count: u32,
    pub height_contrast: f32,
    pub padding: [u32; 3],
    pub scale_rotations: [Vec2; MAX_SURFACE_SCALES],
}

const _: () = assert!(size_of::<SurfaceData>() == 112);

unsafe impl GpuData for SurfaceData {}

impl SurfaceData {
    pub const NONE: SurfaceData = SurfaceData {
        color_texture: 0,
        normal_texture: 0,
        displacement_texture: 0,
        variant_count: 0,
        size: 0.0,
        texels: 0.0,
        displacement_low: 0.0,
        displacement_high: 0.0,
        average_color: Vec3::ZERO,
        scale_count: 0,
        height_contrast: 0.0,
        padding: [0; 3],
        scale_rotations: [Vec2::ZERO; MAX_SURFACE_SCALES],
    };
}
