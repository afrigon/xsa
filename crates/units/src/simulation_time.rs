use std::fmt;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, ensure};
use bitcode::{Decode, Encode};

use crate::civil_date::CivilDate;
use crate::{SECONDS_PER_DAY, SimulationDuration};

const UNIX_EPOCH_SECONDS_SINCE_J2000: f64 = -946_728_000.0;
const SECONDS_PER_HOUR: f64 = 3_600.0;
const SECONDS_PER_MINUTE: f64 = 60.0;
const MILLISECONDS_PER_SECOND: i64 = 1_000;

// Seconds since J2000.0 (2000-01-01T12:00:00Z). UTC is treated as the simulation's time scale; the TT/UTC offset of
// about a minute is ignored.
#[derive(Encode, Decode, Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct SimulationTime {
    pub seconds: f64,
}

impl SimulationTime {
    pub const J2000: SimulationTime = SimulationTime { seconds: 0.0 };

    pub fn now() -> anyhow::Result<SimulationTime> {
        let since_unix_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("the system clock is before 1970")?;

        Ok(SimulationTime {
            seconds: since_unix_epoch.as_secs_f64() + UNIX_EPOCH_SECONDS_SINCE_J2000,
        })
    }

    pub fn seconds_since(self, earlier: SimulationTime) -> f64 {
        self.seconds - earlier.seconds
    }

    pub fn advanced_by(self, duration: SimulationDuration) -> SimulationTime {
        SimulationTime {
            seconds: self.seconds + duration.seconds,
        }
    }

    pub fn is_valid(self) -> bool {
        self.seconds.is_finite()
    }

    fn from_seconds_since_unix_epoch(seconds: f64) -> SimulationTime {
        SimulationTime {
            seconds: seconds + UNIX_EPOCH_SECONDS_SINCE_J2000,
        }
    }

    fn seconds_since_unix_epoch(self) -> f64 {
        self.seconds - UNIX_EPOCH_SECONDS_SINCE_J2000
    }
}

// Accepts `YYYY-MM-DDTHH:MM:SS[.fraction]Z` in UTC.
impl FromStr for SimulationTime {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<SimulationTime> {
        let invalid = || format!("{text:?} is not a UTC timestamp like 2000-01-01T12:00:00Z");
        let without_zone = text.strip_suffix('Z').with_context(invalid)?;
        let (date, clock) = without_zone.split_once('T').with_context(invalid)?;
        let mut date_parts = date.splitn(3, '-');
        let mut clock_parts = clock.splitn(3, ':');
        let mut next_integer = || -> anyhow::Result<i64> { Ok(date_parts.next().with_context(invalid)?.parse()?) };
        let date = CivilDate {
            year: next_integer().with_context(invalid)?,
            month: next_integer().with_context(invalid)?,
            day: next_integer().with_context(invalid)?,
        };
        let mut next_number = || -> anyhow::Result<f64> { Ok(clock_parts.next().with_context(invalid)?.parse()?) };
        let hours = next_number().with_context(invalid)?;
        let minutes = next_number().with_context(invalid)?;
        let seconds = next_number().with_context(invalid)?;
        ensure!(
            (1..=12).contains(&date.month) && (1..=31).contains(&date.day),
            invalid()
        );
        ensure!(hours < 24.0 && minutes < 60.0 && seconds < 61.0, invalid());

        let days = date.days_since_unix_epoch() as f64;

        Ok(SimulationTime::from_seconds_since_unix_epoch(
            days * SECONDS_PER_DAY + hours * SECONDS_PER_HOUR + minutes * SECONDS_PER_MINUTE + seconds,
        ))
    }
}

// Formats as `YYYY-MM-DDTHH:MM:SS.mmmZ`, the inverse of `from_str`.
impl fmt::Display for SimulationTime {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let milliseconds_since_unix_epoch =
            (self.seconds_since_unix_epoch() * MILLISECONDS_PER_SECOND as f64).round() as i64;
        let milliseconds_per_day = SECONDS_PER_DAY as i64 * MILLISECONDS_PER_SECOND;
        let days = milliseconds_since_unix_epoch.div_euclid(milliseconds_per_day);
        let milliseconds_of_day = milliseconds_since_unix_epoch.rem_euclid(milliseconds_per_day);
        let date = CivilDate::from_days_since_unix_epoch(days);
        let seconds_of_day = milliseconds_of_day / MILLISECONDS_PER_SECOND;
        let seconds_per_hour = SECONDS_PER_HOUR as i64;
        let seconds_per_minute = SECONDS_PER_MINUTE as i64;

        write!(
            formatter,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            date.year,
            date.month,
            date.day,
            seconds_of_day / seconds_per_hour,
            seconds_of_day / seconds_per_minute % 60,
            seconds_of_day % seconds_per_minute,
            milliseconds_of_day % MILLISECONDS_PER_SECOND
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> SimulationTime {
        text.parse().unwrap()
    }

    #[test]
    fn formatting_inverts_parsing() {
        for text in [
            "2000-01-01T12:00:00.000Z",
            "1969-07-20T20:17:40.000Z",
            "2024-02-29T23:59:59.500Z",
            "2026-10-02T00:00:00.000Z",
        ] {
            assert_eq!(parse(text).to_string(), text);
        }
    }

    #[test]
    fn j2000_is_zero() {
        assert_eq!(parse("2000-01-01T12:00:00Z"), SimulationTime::J2000);
    }

    #[test]
    fn unix_epoch_matches_the_offset() {
        assert_eq!(parse("1970-01-01T00:00:00Z").seconds, UNIX_EPOCH_SECONDS_SINCE_J2000);
    }

    #[test]
    fn leap_years_are_counted() {
        let seconds = parse("2024-03-01T12:00:00Z").seconds_since(parse("2024-02-28T12:00:00Z"));
        assert_eq!(seconds, 2.0 * SECONDS_PER_DAY);
    }

    #[test]
    fn malformed_timestamps_are_rejected() {
        assert!("2000-13-01T00:00:00Z".parse::<SimulationTime>().is_err());
        assert!("2000-01-01 00:00:00".parse::<SimulationTime>().is_err());
    }
}
