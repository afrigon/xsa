use super::TerrainHandle;
use super::tile_shape::TileShape;

// One instanced draw: a run of a terrain's selected tiles that share a grid size.
pub(in crate::renderer) struct TerrainDraw {
    pub terrain: TerrainHandle,
    pub shape: TileShape,
    pub grid_cells: u32,
    pub first_tile: u32,
    pub tile_count: u32,
}
