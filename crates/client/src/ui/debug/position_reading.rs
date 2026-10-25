use crate::camera::Attitude;

// The camera against the body it is around. Angles in radians; altitudes in meters at the feet, above the ground
// only for a body with terrain.
pub struct PositionReading {
    pub body: String,
    pub latitude: f64,
    pub longitude: f64,
    pub above_ground: Option<f64>,
    pub above_sea: f64,
    pub attitude: Attitude,
}
