use crate::camera::Attitude;

// The camera's attitude against the ecliptic.
pub struct CameraReading {
    pub controls: &'static str,
    // Meters per second, for controls that fly at a set speed.
    pub speed: Option<f64>,
    pub attitude: Attitude,
}
