// Meters relative to the body's radius, signed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::renderer) struct HeightRange {
    pub minimum: f64,
    pub maximum: f64,
}

impl HeightRange {
    pub const SMOOTH: HeightRange = HeightRange {
        minimum: 0.0,
        maximum: 0.0,
    };
}
