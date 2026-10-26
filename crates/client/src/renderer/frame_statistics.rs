use super::TerrainMapStatistics;

#[derive(Clone, Copy, Default)]
pub struct FrameStatistics {
    pub terrain_maps: TerrainMapStatistics,
    pub triangles: u64,
    pub tiles: usize,
    // The finest terrain tiles drawn; None without terrain.
    pub deepest_depth: Option<u32>,
    pub gpu_milliseconds: Option<f32>,
}
