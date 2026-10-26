use std::hash::{Hash, Hasher};

// The range a height map's 16-bit values span, meters relative to the body's radius. Part of the resource's cache key:
// two loads share a resource only with the very same range, so the values compare bit for bit.
#[derive(Clone, Copy, Debug)]
pub struct HeightMapOptions {
    pub minimum: f64,
    pub maximum: f64,
}

impl PartialEq for HeightMapOptions {
    fn eq(&self, other: &HeightMapOptions) -> bool {
        self.minimum.to_bits() == other.minimum.to_bits() && self.maximum.to_bits() == other.maximum.to_bits()
    }
}

impl Eq for HeightMapOptions {}

impl Hash for HeightMapOptions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.minimum.to_bits().hash(state);
        self.maximum.to_bits().hash(state);
    }
}
