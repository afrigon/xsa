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
