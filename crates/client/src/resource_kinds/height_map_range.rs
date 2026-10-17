use std::hash::{Hash, Hasher};

// The heights a height map's values span, meters relative to the body's radius. Equal when their bits are, so it can
// key a cached resource.
#[derive(Clone, Copy, Debug)]
pub struct HeightMapRange {
    pub minimum: f64,
    pub maximum: f64,
}

impl PartialEq for HeightMapRange {
    fn eq(&self, other: &HeightMapRange) -> bool {
        self.minimum.to_bits() == other.minimum.to_bits() && self.maximum.to_bits() == other.maximum.to_bits()
    }
}

impl Eq for HeightMapRange {}

impl Hash for HeightMapRange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.minimum.to_bits().hash(state);
        self.maximum.to_bits().hash(state);
    }
}
