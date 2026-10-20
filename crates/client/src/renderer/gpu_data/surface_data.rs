use glam::Vec3;

use super::GpuData;

// One class's surface as the terrain programs read it. Mirrored by `SurfaceData` in terrain/surface/surface-data.slang.
// Sizes are in meters. Displacements are compared across the two blended materials, the higher showing through by
// `height_contrast` of the 0 to 1 displacement range. The texture is laid on hexagonal cells `hex_cell_size` repeats apart, each turning and shifting the texture by its id hashed with
// `tiling_seed`.
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
    pub height_contrast: f32,
    pub hex_cell_size: f32,
    pub tiling_seed: u32,
    pub padding: [u32; 2],
}

const _: () = assert!(size_of::<SurfaceData>() == 64);

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
        height_contrast: 0.0,
        hex_cell_size: 0.0,
        tiling_seed: 0,
        padding: [0; 2],
    };
}
