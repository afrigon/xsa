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

    // The range moved `margin` meters further each way.
    pub fn widened(self, margin: f64) -> HeightRange {
        HeightRange {
            minimum: self.minimum - margin,
            maximum: self.maximum + margin,
        }
    }

    pub fn scaled(self, scale: f64) -> HeightRange {
        HeightRange {
            minimum: self.minimum * scale,
            maximum: self.maximum * scale,
        }
    }
}
