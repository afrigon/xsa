use super::{ScatterStatistics, TerrainStatistics};

#[derive(Clone, Copy, Default)]
pub struct FrameStatistics {
    pub terrain: TerrainStatistics,
    pub scatter: ScatterStatistics,
    pub gpu_milliseconds: Option<f32>,
}
