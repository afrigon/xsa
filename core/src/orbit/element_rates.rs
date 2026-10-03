#[derive(Clone, Copy, Debug, Default)]
pub struct ElementRates {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub ascending_node: f64,
    pub periapsis: f64,
}
