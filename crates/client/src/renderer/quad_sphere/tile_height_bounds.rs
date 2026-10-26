use super::{HeightRange, QuadSphereTile};

// What tile selection knows of a terrain's heights, in meters relative to the radius: every height it has, and every
// height a tile can show at any detail it is drawn at. None while it does not know them yet.
pub(in crate::renderer) trait TileHeightBounds {
    fn overall_range(&self) -> Option<HeightRange>;

    fn tile_range(&self, tile: QuadSphereTile) -> Option<HeightRange>;
}
