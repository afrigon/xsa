use std::fmt;

use bitcode::{Decode, Encode};

use crate::{TimeRate, TimeWarp};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq)]
pub struct TimeFlow {
    pub rate: TimeRate,
    pub time_override: Option<TimeRate>,
    // Runs time ahead while set; an override pauses it.
    pub warp: Option<TimeWarp>,
}

impl TimeFlow {
    pub const REAL_TIME: TimeFlow = TimeFlow {
        rate: TimeRate::REAL_TIME,
        time_override: None,
        warp: None,
    };

    // The rate set to run at, the override first; a warp runs faster for a while.
    pub fn effective(self) -> TimeRate {
        self.time_override.unwrap_or(self.rate)
    }
}

impl fmt::Display for TimeFlow {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.rate)?;

        if let Some(time_override) = self.time_override {
            write!(formatter, ", override {time_override}")?;
        }

        if let Some(warp) = self.warp {
            write!(formatter, ", warping to {}", warp.until)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_override_wins_over_the_rate() {
        let flow = TimeFlow {
            rate: TimeRate { multiplier: 8.0 },
            time_override: Some(TimeRate::PAUSED),
            warp: None,
        };
        assert_eq!(flow.effective(), TimeRate::PAUSED);
        assert_eq!(TimeFlow::REAL_TIME.effective(), TimeRate::REAL_TIME);
    }

    #[test]
    fn shows_the_override_after_the_rate() {
        let flow = TimeFlow {
            rate: TimeRate { multiplier: 8.0 },
            time_override: Some(TimeRate::PAUSED),
            warp: None,
        };
        assert_eq!(flow.to_string(), "rate 8×, override paused");
    }
}
