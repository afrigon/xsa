use glam::Vec3;

use super::GpuData;

// Centers are relative to the camera of their frame; the cube center is on the cube spanning -1 to 1, the axes are its
// face's unit column and row axes, and the half size is in the face's equal-angle coordinates. `face` (by `CubeFace`
// index), `x`, `y` and `depth` are the tile's place in its face's quadtree, which its heights are read at.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TileData {
    pub center: Vec3,
    pub detail: i32,
    pub previous_center: Vec3,
    pub grid_cells: u32,
    pub cube_center: Vec3,
    pub morph_start: f32,
    pub column_axis: Vec3,
    pub morph_end: f32,
    pub row_axis: Vec3,
    pub half_size: f32,
    pub x: u32,
    pub y: u32,
    pub depth: u32,
    pub face: u32,
}

const _: () = assert!(size_of::<TileData>() == 96);

unsafe impl GpuData for TileData {}
