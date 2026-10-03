use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeRate {
    pub multiplier: f64,
}

impl TimeRate {
    pub const PAUSED: TimeRate = TimeRate { multiplier: 0.0 };
    pub const REAL_TIME: TimeRate = TimeRate { multiplier: 1.0 };

    pub fn is_paused(self) -> bool {
        self.multiplier == 0.0
    }

    pub fn is_valid(self) -> bool {
        self.multiplier.is_finite() && self.multiplier >= 0.0
    }
}

impl fmt::Display for TimeRate {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.is_paused() {
            write!(formatter, "paused")
        } else {
            write!(formatter, "rate {}×", self.multiplier)
        }
    }
}
