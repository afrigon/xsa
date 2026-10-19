use std::time::Duration;

struct TimeUnit {
    name: &'static str,
    milliseconds: u128,
}

// Largest first, so a duration is written in the largest unit that holds it exactly.
const UNITS: [TimeUnit; 3] = [
    TimeUnit {
        name: "min",
        milliseconds: 60_000,
    },
    TimeUnit {
        name: "s",
        milliseconds: 1000,
    },
    TimeUnit {
        name: "ms",
        milliseconds: 1,
    },
];

// A duration written with a unit in the config (`50ms`, `0.2s`, `1min`).
pub struct DurationName;

impl DurationName {
    pub fn parse(name: &str) -> Option<Duration> {
        UNITS.iter().find_map(|unit| {
            let count = name.strip_suffix(unit.name)?.parse::<f64>().ok()?;
            let seconds = count * unit.milliseconds as f64 / 1000.0;

            Duration::try_from_secs_f64(seconds).ok()
        })
    }

    pub fn write(duration: Duration) -> String {
        let milliseconds = duration.as_millis();
        let exact = Duration::from_millis(milliseconds as u64) == duration;
        let unit = UNITS
            .iter()
            .find(|unit| exact && milliseconds >= unit.milliseconds && milliseconds.is_multiple_of(unit.milliseconds));

        match unit {
            Some(unit) => format!("{}{}", milliseconds / unit.milliseconds, unit.name),
            None => format!("{}ms", duration.as_secs_f64() * 1000.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_unit_and_decimals() {
        assert_eq!(DurationName::parse("50ms"), Some(Duration::from_millis(50)));
        assert_eq!(DurationName::parse("0.2s"), Some(Duration::from_millis(200)));
        assert_eq!(DurationName::parse("2min"), Some(Duration::from_secs(120)));
    }

    #[test]
    fn rejects_a_missing_unit_or_a_negative_duration() {
        assert_eq!(DurationName::parse("50"), None);
        assert_eq!(DurationName::parse("ms"), None);
        assert_eq!(DurationName::parse("-1s"), None);
    }

    #[test]
    fn writes_the_largest_exact_unit() {
        assert_eq!(DurationName::write(Duration::from_secs(120)), "2min");
        assert_eq!(DurationName::write(Duration::from_millis(1500)), "1500ms");
        assert_eq!(DurationName::write(Duration::from_secs(3)), "3s");
        assert_eq!(DurationName::write(Duration::ZERO), "0ms");
    }
}
