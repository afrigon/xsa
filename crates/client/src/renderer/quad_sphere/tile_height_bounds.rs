use super::{HeightRange, QuadSphereTile};

// What tile selection knows of a terrain's heights, in meters relative to the radius: every height it has, and the
// heights a tile can show when none of its vertices is farther than `farthest` meters from the camera. None while it
// does not know them yet.
pub(in crate::renderer) trait TileHeightBounds {
    fn overall_range(&self) -> Option<HeightRange>;

    fn tile_range(&self, tile: QuadSphereTile, farthest: f64) -> Option<HeightRange>;
}
