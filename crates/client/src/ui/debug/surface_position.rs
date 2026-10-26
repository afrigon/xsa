use crate::camera::Attitude;

// Where on its body the camera is, which only exists away from the body's centre. Angles in radians; the altitude in
// meters at the feet, only for a body with terrain.
pub struct SurfacePosition {
    pub latitude: f64,
    pub longitude: f64,
    pub above_ground: Option<f64>,
    pub attitude: Attitude,
}
