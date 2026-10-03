use std::str::FromStr;

use xsa_units::SimulationDuration;

struct TimeUnit {
    suffix: &'static str,
    seconds: f64,
}

const TIME_UNITS: [TimeUnit; 3] = [
    TimeUnit {
        suffix: "min",
        seconds: 60.0,
    },
    TimeUnit {
        suffix: "h",
        seconds: 3_600.0,
    },
    TimeUnit {
        suffix: "s",
        seconds: 1.0,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Duration {
    Seconds { seconds: f64 },
    Ticks { ticks: u64 },
}

impl Duration {
    pub fn duration(self) -> SimulationDuration {
        match self {
            Duration::Seconds { seconds } => SimulationDuration { seconds },
            Duration::Ticks { ticks } => SimulationDuration::from_ticks(ticks),
        }
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("{text:?} is not a duration like 10s, 5min, 2h or 60t (ticks)");
        if let Some(ticks) = text.strip_suffix('t') {
            let ticks = ticks.parse().map_err(|_| invalid())?;
            return Ok(Duration::Ticks { ticks });
        }
        let unit = TIME_UNITS
            .iter()
            .find(|unit| text.ends_with(unit.suffix))
            .ok_or_else(invalid)?;
        let value: f64 = text[..text.len() - unit.suffix.len()].parse().map_err(|_| invalid())?;
        if !value.is_finite() {
            return Err(invalid());
        }
        Ok(Duration::Seconds {
            seconds: value * unit.seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use xsa_units::TICK_RATE_HERTZ;

    use super::*;

    fn seconds(text: &str) -> f64 {
        text.parse::<Duration>().unwrap().duration().seconds
    }

    #[test]
    fn units_convert_to_seconds() {
        assert_eq!(seconds("10s"), 10.0);
        assert_eq!(seconds("1.5min"), 90.0);
        assert_eq!(seconds("2h"), 7_200.0);
        assert_eq!(seconds("1e3s"), 1_000.0);
    }

    #[test]
    fn ticks_follow_the_tick_rate() {
        let duration: Duration = "60t".parse().unwrap();
        assert_eq!(duration, Duration::Ticks { ticks: 60 });
        assert_eq!(duration.duration().seconds, 60.0 / TICK_RATE_HERTZ);
    }

    #[test]
    fn malformed_durations_are_rejected() {
        for text in ["", "10", "s", "1.5t", "10d", "infs", "NaNs", "10 s"] {
            assert!(text.parse::<Duration>().is_err(), "{text:?}");
        }
    }
}
