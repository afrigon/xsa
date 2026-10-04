use super::{BodyCategory, BodyId, Star};

pub struct BodyDescription {
    pub id: BodyId,
    pub category: BodyCategory,
    pub radius: f64,
    pub gravitational_parameter: f64,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub star: Option<Star>,
}
