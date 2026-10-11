#[derive(Clone, Copy, Default)]
pub struct FrameStatistics {
    pub triangles: u64,
    pub ev100: f32,
    pub anchor_ev100: f32,
}
