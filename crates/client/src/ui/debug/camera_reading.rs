use crate::camera::{Attitude, CameraMode};

// The camera's attitude against the ecliptic.
pub struct CameraReading {
    pub mode: CameraMode,
    // Meters per second; the debug camera's only.
    pub speed: Option<f64>,
    pub attitude: Attitude,
}
