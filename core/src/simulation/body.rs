use super::{BodyId, Star};

pub struct Body {
    pub id: BodyId,
    pub radius: f64,
    pub gravitational_parameter: f64,
    pub star: Option<Star>,
}
