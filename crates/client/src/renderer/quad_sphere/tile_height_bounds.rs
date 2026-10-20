use super::{HeightRange, QuadSphereTile};

// What tile selection knows of a terrain's heights: the range of the whole body, and of the heights each tile's
// vertices can take. None while it does not know them yet.
pub(in crate::renderer) trait TileHeightBounds {
    fn overall_range(&self) -> Option<HeightRange>;

    fn tile_range(&self, tile: QuadSphereTile) -> Option<HeightRange>;
}
