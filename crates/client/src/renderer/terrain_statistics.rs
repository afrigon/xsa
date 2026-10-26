#[derive(Clone, Copy, Default)]
pub struct TerrainStatistics {
    pub triangles: u64,
    pub tiles: usize,
    // The finest tiles drawn; None without terrain.
    pub deepest_depth: Option<u32>,
}
