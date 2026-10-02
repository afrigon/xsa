use std::str::FromStr;

use xsa_core::time::TICK_RATE_HERTZ;

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
    pub fn seconds(self) -> f64 {
        match self {
            Duration::Seconds { seconds } => seconds,
            Duration::Ticks { ticks } => ticks as f64 / TICK_RATE_HERTZ,
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
    use super::*;

    #[test]
    fn units_convert_to_seconds() {
        assert_eq!("10s".parse::<Duration>().unwrap().seconds(), 10.0);
        assert_eq!("1.5min".parse::<Duration>().unwrap().seconds(), 90.0);
        assert_eq!("2h".parse::<Duration>().unwrap().seconds(), 7_200.0);
        assert_eq!("1e3s".parse::<Duration>().unwrap().seconds(), 1_000.0);
    }

    #[test]
    fn ticks_follow_the_tick_rate() {
        let duration: Duration = "60t".parse().unwrap();
        assert_eq!(duration, Duration::Ticks { ticks: 60 });
        assert_eq!(duration.seconds(), 60.0 / TICK_RATE_HERTZ);
    }

    #[test]
    fn malformed_durations_are_rejected() {
        for text in ["", "10", "s", "1.5t", "10d", "infs", "NaNs", "10 s"] {
            assert!(text.parse::<Duration>().is_err(), "{text:?}");
        }
    }
}
