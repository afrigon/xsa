// The lowest and highest stored value of a set of height texels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TexelRange {
    pub minimum: u16,
    pub maximum: u16,
}

impl TexelRange {
    pub const EMPTY: TexelRange = TexelRange {
        minimum: u16::MAX,
        maximum: u16::MIN,
    };

    pub fn including(self, value: u16) -> TexelRange {
        TexelRange {
            minimum: self.minimum.min(value),
            maximum: self.maximum.max(value),
        }
    }

    // What both bound; both hold every value, so the narrower of each side does too.
    pub fn intersection(self, other: TexelRange) -> TexelRange {
        TexelRange {
            minimum: self.minimum.max(other.minimum),
            maximum: self.maximum.min(other.maximum),
        }
    }

    pub fn union(self, other: TexelRange) -> TexelRange {
        TexelRange {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }
}
