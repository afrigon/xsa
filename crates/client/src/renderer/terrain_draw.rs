use super::TerrainHandle;
use super::shading_range::ShadingRange;
use super::tile_shape::TileShape;

// One instanced draw: a run of a terrain's selected tiles that share a shading range and a grid size.
pub(in crate::renderer) struct TerrainDraw {
    pub terrain: TerrainHandle,
    pub shape: TileShape,
    pub range: ShadingRange,
    pub grid_cells: u32,
    pub first_tile: u32,
    pub tile_count: u32,
}
