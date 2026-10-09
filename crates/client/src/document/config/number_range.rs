use std::fmt;

// Inclusive; an unbounded side is infinite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumberRange {
    minimum: f32,
    maximum: f32,
}

impl NumberRange {
    pub const ANY: NumberRange = NumberRange {
        minimum: f32::NEG_INFINITY,
        maximum: f32::INFINITY,
    };

    pub const fn at_least(minimum: f32) -> NumberRange {
        NumberRange {
            minimum,
            maximum: f32::INFINITY,
        }
    }

    pub const fn between(minimum: f32, maximum: f32) -> NumberRange {
        NumberRange { minimum, maximum }
    }

    pub fn contains(self, number: f32) -> bool {
        (self.minimum..=self.maximum).contains(&number)
    }
}

impl Default for NumberRange {
    fn default() -> NumberRange {
        NumberRange::ANY
    }
}

impl fmt::Display for NumberRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match (self.minimum.is_finite(), self.maximum.is_finite()) {
            (true, true) => write!(formatter, "{} to {}", self.minimum, self.maximum),
            (true, false) => write!(formatter, "{} or more", self.minimum),
            (false, true) => write!(formatter, "{} or less", self.maximum),
            (false, false) => write!(formatter, "any number"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_are_inclusive() {
        let range = NumberRange::between(0.0, 1.0);
        assert!(range.contains(0.0) && range.contains(1.0));
        assert!(!range.contains(-0.1) && !range.contains(1.1));
    }

    #[test]
    fn an_open_side_has_no_limit() {
        assert!(NumberRange::at_least(0.0).contains(f32::MAX));
        assert!(!NumberRange::at_least(0.0).contains(-1.0));
        assert!(NumberRange::ANY.contains(f32::MIN));
    }
}
