use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, ensure};

const SECONDS_PER_DAY: f64 = 86_400.0;
const UNIX_EPOCH_SECONDS_SINCE_J2000: f64 = -946_728_000.0;

pub const SECONDS_PER_JULIAN_CENTURY: f64 = 36_525.0 * SECONDS_PER_DAY;
pub const TICK_RATE_HERTZ: f64 = 60.0;

pub fn now() -> anyhow::Result<f64> {
    let since_unix_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("the system clock is before 1970")?;
    Ok(since_unix_epoch.as_secs_f64() + UNIX_EPOCH_SECONDS_SINCE_J2000)
}

// Accepts `YYYY-MM-DDTHH:MM:SS[.fraction]Z` in UTC and returns seconds since J2000.0
// (2000-01-01T12:00:00Z; the TT/UTC offset of about a minute is ignored).
pub fn parse_timestamp(text: &str) -> anyhow::Result<f64> {
    let invalid = || format!("{text:?} is not a UTC timestamp like 2000-01-01T12:00:00Z");
    let without_zone = text.strip_suffix('Z').with_context(invalid)?;
    let (date, clock) = without_zone.split_once('T').with_context(invalid)?;
    let mut date_parts = date.splitn(3, '-');
    let year: i64 = date_parts.next().with_context(invalid)?.parse().with_context(invalid)?;
    let month: i64 = date_parts.next().with_context(invalid)?.parse().with_context(invalid)?;
    let day: i64 = date_parts.next().with_context(invalid)?.parse().with_context(invalid)?;
    let mut clock_parts = clock.splitn(3, ':');
    let hours: f64 = clock_parts
        .next()
        .with_context(invalid)?
        .parse()
        .with_context(invalid)?;
    let minutes: f64 = clock_parts
        .next()
        .with_context(invalid)?
        .parse()
        .with_context(invalid)?;
    let seconds: f64 = clock_parts
        .next()
        .with_context(invalid)?
        .parse()
        .with_context(invalid)?;
    ensure!((1..=12).contains(&month) && (1..=31).contains(&day), invalid());
    ensure!(hours < 24.0 && minutes < 60.0 && seconds < 61.0, invalid());

    let days_since_unix_epoch = days_from_civil(year, month, day) as f64;
    let seconds_since_unix_epoch = days_since_unix_epoch * SECONDS_PER_DAY + hours * 3_600.0 + minutes * 60.0 + seconds;
    Ok(seconds_since_unix_epoch + UNIX_EPOCH_SECONDS_SINCE_J2000)
}

// Formats seconds since J2000.0 as `YYYY-MM-DDTHH:MM:SS.mmmZ`, the inverse of `parse_timestamp`.
pub fn format_timestamp(time: f64) -> String {
    let milliseconds_since_unix_epoch = ((time - UNIX_EPOCH_SECONDS_SINCE_J2000) * 1_000.0).round() as i64;
    let milliseconds_per_day = SECONDS_PER_DAY as i64 * 1_000;
    let days = milliseconds_since_unix_epoch.div_euclid(milliseconds_per_day);
    let milliseconds_of_day = milliseconds_since_unix_epoch.rem_euclid(milliseconds_per_day);
    let date = civil_from_days(days);
    let seconds_of_day = milliseconds_of_day / 1_000;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.year,
        date.month,
        date.day,
        seconds_of_day / 3_600,
        seconds_of_day / 60 % 60,
        seconds_of_day % 60,
        milliseconds_of_day % 1_000
    )
}

struct CivilDate {
    year: i64,
    month: i64,
    day: i64,
}

// Howard Hinnant's civil_from_days, the inverse of days_from_civil.
fn civil_from_days(days_since_unix_epoch: i64) -> CivilDate {
    let days = days_since_unix_epoch + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    CivilDate { year, month, day }
}

// Howard Hinnant's days_from_civil: days since 1970-01-01 in the proleptic Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_inverts_parsing() {
        for text in [
            "2000-01-01T12:00:00.000Z",
            "1969-07-20T20:17:40.000Z",
            "2024-02-29T23:59:59.500Z",
            "2026-10-02T00:00:00.000Z",
        ] {
            assert_eq!(format_timestamp(parse_timestamp(text).unwrap()), text);
        }
    }

    #[test]
    fn j2000_is_zero() {
        assert_eq!(parse_timestamp("2000-01-01T12:00:00Z").unwrap(), 0.0);
    }

    #[test]
    fn unix_epoch_matches_the_offset() {
        assert_eq!(
            parse_timestamp("1970-01-01T00:00:00Z").unwrap(),
            UNIX_EPOCH_SECONDS_SINCE_J2000
        );
    }

    #[test]
    fn leap_years_are_counted() {
        let seconds =
            parse_timestamp("2024-03-01T12:00:00Z").unwrap() - parse_timestamp("2024-02-28T12:00:00Z").unwrap();
        assert_eq!(seconds, 2.0 * SECONDS_PER_DAY);
    }

    #[test]
    fn malformed_timestamps_are_rejected() {
        assert!(parse_timestamp("2000-13-01T00:00:00Z").is_err());
        assert!(parse_timestamp("2000-01-01 00:00:00").is_err());
    }
}
