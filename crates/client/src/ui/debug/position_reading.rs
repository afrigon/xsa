use super::SurfacePosition;

// The camera against the body it is around. The altitude in meters at the feet; no surface position at the body's
// centre.
pub struct PositionReading {
    pub body: String,
    pub above_sea: f64,
    pub surface: Option<SurfacePosition>,
}
