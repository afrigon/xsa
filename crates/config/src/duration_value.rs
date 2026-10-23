use std::time::Duration;

use anyhow::Context;
use kdl::KdlValue;

use crate::ConfigValue;

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

// A duration is written with a unit in the config (`50ms`, `0.2s`, `1min`).
impl ConfigValue for Duration {
    fn from_kdl(value: &KdlValue) -> Option<Duration> {
        value.as_string().and_then(duration_from_name)
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::String(duration_name(*self))
    }

    fn parse(text: &str) -> anyhow::Result<Duration> {
        duration_from_name(text).with_context(|| format!("{text:?} is not a duration like 50ms, 0.2s or 1min"))
    }
}

fn duration_from_name(name: &str) -> Option<Duration> {
    UNITS.iter().find_map(|unit| {
        let count = name.strip_suffix(unit.name)?.parse::<f64>().ok()?;
        let seconds = count * unit.milliseconds as f64 / 1000.0;

        Duration::try_from_secs_f64(seconds).ok()
    })
}

fn duration_name(duration: Duration) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_unit_and_decimals() {
        assert_eq!(duration_from_name("50ms"), Some(Duration::from_millis(50)));
        assert_eq!(duration_from_name("0.2s"), Some(Duration::from_millis(200)));
        assert_eq!(duration_from_name("2min"), Some(Duration::from_secs(120)));
    }

    #[test]
    fn rejects_a_missing_unit_or_a_negative_duration() {
        assert_eq!(duration_from_name("50"), None);
        assert_eq!(duration_from_name("ms"), None);
        assert_eq!(duration_from_name("-1s"), None);
    }

    #[test]
    fn writes_the_largest_exact_unit() {
        assert_eq!(duration_name(Duration::from_secs(120)), "2min");
        assert_eq!(duration_name(Duration::from_millis(1500)), "1500ms");
        assert_eq!(duration_name(Duration::from_secs(3)), "3s");
        assert_eq!(duration_name(Duration::ZERO), "0ms");
    }
}
