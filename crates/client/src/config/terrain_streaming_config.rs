#[derive(Clone, Debug, PartialEq)]
pub struct TerrainStreamingConfig {
    // Terrain pages stay as they are: nothing loads or is evicted while tiles keep requesting.
    pub freeze: bool,
}
