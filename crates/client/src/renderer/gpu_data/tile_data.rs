use glam::{IVec2, Vec2, Vec3};

use super::GpuData;

// Centers are relative to the camera of their frame; cube coordinates and axes are on the cube spanning -1 to 1.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TileData {
    pub center: Vec3,
    pub detail: i32,
    pub previous_center: Vec3,
    pub grid_cells: u32,
    pub cube_center: Vec3,
    pub morph_start: f32,
    pub horizontal_axis: Vec3,
    pub morph_end: f32,
    pub vertical_axis: Vec3,
    pub padding: u32,
    // The center's position on the height map's top level, split so the fraction keeps full precision.
    pub center_texel: IVec2,
    pub center_texel_fraction: Vec2,
}

const _: () = assert!(size_of::<TileData>() == 96);

unsafe impl GpuData for TileData {}
