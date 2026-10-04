use super::{BodyCategory, BodyId, BodyIndex, Star};

pub struct Body {
    pub id: BodyId,
    pub category: BodyCategory,
    // The body this one orbits; a barycenter's secondary and the bodies orbiting a barycenter belong to its primary.
    pub parent: Option<BodyIndex>,
    pub radius: f64,
    pub gravitational_parameter: f64,
    pub star: Option<Star>,
}
