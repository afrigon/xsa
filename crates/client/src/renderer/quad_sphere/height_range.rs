use resource_loader::terrain::HeightRange as PageHeightRange;

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

    pub fn union(self, other: HeightRange) -> HeightRange {
        HeightRange {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }
}

impl From<PageHeightRange> for HeightRange {
    fn from(range: PageHeightRange) -> HeightRange {
        HeightRange {
            minimum: f64::from(range.minimum),
            maximum: f64::from(range.maximum),
        }
    }
}
