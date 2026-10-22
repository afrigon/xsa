use std::str::FromStr;

use super::length_unit::LENGTH_UNITS;

const PER_SECOND_SUFFIX: &str = "/s";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speed {
    pub meters_per_second: f64,
}

impl FromStr for Speed {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid =
            || format!("{text:?} is not a speed like 100m/s (units: m/s, km/s, Mm/s, Gm/s; a bare number is m/s)");
        let (number, scale) = match text.strip_suffix(PER_SECOND_SUFFIX) {
            Some(distance) => {
                let unit = LENGTH_UNITS
                    .iter()
                    .find(|unit| distance.ends_with(unit.suffix))
                    .ok_or_else(invalid)?;

                (&distance[..distance.len() - unit.suffix.len()], unit.meters)
            }
            None => (text, 1.0),
        };
        let value: f64 = number.parse().map_err(|_| invalid())?;

        if !value.is_finite() {
            return Err(invalid());
        }

        Ok(Speed {
            meters_per_second: value * scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meters_per_second(text: &str) -> f64 {
        text.parse::<Speed>().unwrap().meters_per_second
    }

    #[test]
    fn units_convert_to_meters_per_second() {
        assert_eq!(meters_per_second("100m/s"), 100.0);
        assert_eq!(meters_per_second("7.8km/s"), 7_800.0);
        assert_eq!(meters_per_second("1.5Mm/s"), 1_500_000.0);
        assert_eq!(meters_per_second("2Gm/s"), 2e9);
    }

    #[test]
    fn a_bare_number_is_meters_per_second() {
        assert_eq!(meters_per_second("100"), 100.0);
    }

    #[test]
    fn a_speed_may_be_zero_or_negative() {
        assert_eq!(meters_per_second("0m/s"), 0.0);
        assert_eq!(meters_per_second("-25m/s"), -25.0);
    }

    #[test]
    fn malformed_speeds_are_rejected() {
        for text in ["", "m/s", "100m", "100/s", "1mm/s", "1KM/s", "infm/s", "1 m/s", "fast"] {
            assert!(text.parse::<Speed>().is_err(), "{text:?}");
        }
    }
}
