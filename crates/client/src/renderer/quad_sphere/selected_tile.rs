use super::QuadSphereTile;

// A tile chosen for drawing. Its detail level is its depth, or below zero for a root drawn with a coarser grid.
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct SelectedTile {
    pub tile: QuadSphereTile,
    pub detail: i32,
}
