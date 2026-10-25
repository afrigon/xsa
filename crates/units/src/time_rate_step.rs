use bitcode::{Decode, Encode};

use crate::TimeRate;

const STEP_FACTOR: f64 = 2.0;

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq)]
pub enum TimeRateStep {
    // Steps up from a pause through slow motion to real time, then doubles the rate.
    Faster,
    // Halves the rate down to real time, then steps down through slow motion and pauses.
    Slower,
}

impl TimeRateStep {
    pub fn apply(self, current: TimeRate) -> TimeRate {
        match self {
            TimeRateStep::Faster => Self::faster(current),
            TimeRateStep::Slower => Self::slower(current),
        }
    }

    fn faster(current: TimeRate) -> TimeRate {
        if current >= TimeRate::REAL_TIME {
            return TimeRate {
                multiplier: current.multiplier * STEP_FACTOR,
            };
        }

        TimeRate::SLOW_MOTION
            .into_iter()
            .rev()
            .find(|step| *step > current)
            .unwrap_or(TimeRate::REAL_TIME)
    }

    fn slower(current: TimeRate) -> TimeRate {
        let halved = TimeRate {
            multiplier: current.multiplier / STEP_FACTOR,
        };

        if halved >= TimeRate::REAL_TIME {
            return halved;
        }

        TimeRate::SLOW_MOTION
            .into_iter()
            .find(|step| *step < current)
            .unwrap_or(TimeRate::PAUSED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(multiplier: f64) -> TimeRate {
        TimeRate { multiplier }
    }

    #[test]
    fn faster_doubles_from_real_time() {
        assert_eq!(TimeRateStep::Faster.apply(rate(4.0)), rate(8.0));
    }

    #[test]
    fn faster_steps_up_from_a_pause_through_slow_motion_to_real_time() {
        assert_eq!(TimeRateStep::Faster.apply(TimeRate::PAUSED), rate(0.1));
        assert_eq!(TimeRateStep::Faster.apply(rate(0.1)), rate(0.25));
        assert_eq!(TimeRateStep::Faster.apply(rate(0.25)), rate(0.5));
        assert_eq!(TimeRateStep::Faster.apply(rate(0.5)), TimeRate::REAL_TIME);
        assert_eq!(TimeRateStep::Faster.apply(rate(0.3)), rate(0.5));
        assert_eq!(TimeRateStep::Faster.apply(rate(0.05)), rate(0.1));
    }

    #[test]
    fn slower_halves_down_to_real_time() {
        assert_eq!(TimeRateStep::Slower.apply(rate(8.0)), rate(4.0));
        assert_eq!(TimeRateStep::Slower.apply(rate(2.0)), TimeRate::REAL_TIME);
    }

    #[test]
    fn slower_steps_down_through_slow_motion_then_pauses() {
        assert_eq!(TimeRateStep::Slower.apply(TimeRate::REAL_TIME), rate(0.5));
        assert_eq!(TimeRateStep::Slower.apply(rate(0.5)), rate(0.25));
        assert_eq!(TimeRateStep::Slower.apply(rate(0.25)), rate(0.1));
        assert_eq!(TimeRateStep::Slower.apply(rate(0.1)), TimeRate::PAUSED);
        assert_eq!(TimeRateStep::Slower.apply(rate(1.5)), rate(0.5));
        assert_eq!(TimeRateStep::Slower.apply(rate(0.3)), rate(0.25));
        assert_eq!(TimeRateStep::Slower.apply(TimeRate::PAUSED), TimeRate::PAUSED);
    }
}
