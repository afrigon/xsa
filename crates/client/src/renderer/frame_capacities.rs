// How many items each per-frame buffer holds; a buffer outgrown is created again at the next power of two.
#[derive(Clone, Copy)]
pub(super) struct FrameCapacities {
    pub objects: usize,
    pub tiles: usize,
    pub terrains: usize,
    pub stars: usize,
    pub lights: usize,
    pub surface_anchors: usize,
    pub terrain_draws: usize,
    pub mesh_draws: usize,
}
