use xsa_core::terrain::QuadSphereTile;

// A tile chosen for drawing. Its detail level is its depth, or below zero for a root drawn with a coarser grid. Its
// nearest point is this far from the camera, in meters.
#[derive(Clone, Copy, Debug)]
pub(in crate::renderer) struct SelectedTile {
    pub tile: QuadSphereTile,
    pub detail: i32,
    pub nearest: f64,
}
