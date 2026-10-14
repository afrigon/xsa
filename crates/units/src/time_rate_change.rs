use bitcode::{Decode, Encode};

use crate::TimeRate;

const STEP_FACTOR: f64 = 2.0;

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq)]
pub enum TimeRateChange {
    To(TimeRate),
    // Doubles the rate; a paused simulation stays paused.
    Faster,
    // Halves the rate, pausing once it would fall below real time.
    Slower,
}

impl TimeRateChange {
    pub fn apply(self, current: TimeRate) -> TimeRate {
        match self {
            TimeRateChange::To(rate) => rate,
            TimeRateChange::Faster => TimeRate {
                multiplier: current.multiplier * STEP_FACTOR,
            },
            TimeRateChange::Slower => {
                let multiplier = current.multiplier / STEP_FACTOR;

                if multiplier < TimeRate::REAL_TIME.multiplier {
                    TimeRate::PAUSED
                } else {
                    TimeRate { multiplier }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(multiplier: f64) -> TimeRate {
        TimeRate { multiplier }
    }

    #[test]
    fn to_sets_the_rate() {
        assert_eq!(TimeRateChange::To(rate(5.0)).apply(rate(2.0)), rate(5.0));
    }

    #[test]
    fn faster_doubles_and_keeps_a_pause() {
        assert_eq!(TimeRateChange::Faster.apply(rate(4.0)), rate(8.0));
        assert_eq!(TimeRateChange::Faster.apply(TimeRate::PAUSED), TimeRate::PAUSED);
    }

    #[test]
    fn slower_halves_down_to_real_time_then_pauses() {
        assert_eq!(TimeRateChange::Slower.apply(rate(8.0)), rate(4.0));
        assert_eq!(TimeRateChange::Slower.apply(rate(2.0)), TimeRate::REAL_TIME);
        assert_eq!(TimeRateChange::Slower.apply(TimeRate::REAL_TIME), TimeRate::PAUSED);
        assert_eq!(TimeRateChange::Slower.apply(rate(1.5)), TimeRate::PAUSED);
    }
}
