use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

// Below a straight angle: a perspective projection cannot reach its sides.
const FIELD_OF_VIEW_DEGREES: NumberRange = NumberRange::between(1.0, 179.0);
const ZOOM: NumberRange = NumberRange::at_least(1.0);

#[derive(Clone, Debug, PartialEq)]
pub struct CameraConfig {
    // Horizontal, in degrees.
    pub fov: f32,
    // How many times the view magnifies: the focal length grows by it, so the field of view narrows.
    pub zoom: f32,
}

impl CameraConfig {
    // The tangent of half the horizontal field of view, zoom included.
    pub fn half_angle_tangent(&self) -> f64 {
        (f64::from(self.fov).to_radians() / 2.0).tan() / f64::from(self.zoom)
    }
}

impl ConfigSection for CameraConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.number("fov", &mut self.fov, FIELD_OF_VIEW_DEGREES);
        visitor.number("zoom", &mut self.zoom, ZOOM);
    }
}
