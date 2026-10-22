use std::str::FromStr;

use super::length_unit::{LENGTH_UNITS, METERS_PER_KILOMETER};

// Meters of the feet above the ground, below it when negative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Altitude {
    pub meters: f64,
}

impl FromStr for Altitude {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("{text:?} is not an altitude like 100m (units: m, km, Mm, Gm; a bare number is km)");
        let unit = LENGTH_UNITS.iter().find(|unit| text.ends_with(unit.suffix));
        let number = unit.map_or(text, |unit| &text[..text.len() - unit.suffix.len()]);
        let scale = unit.map_or(METERS_PER_KILOMETER, |unit| unit.meters);
        let value: f64 = number.parse().map_err(|_| invalid())?;

        if !value.is_finite() {
            return Err(invalid());
        }

        Ok(Altitude { meters: value * scale })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meters(text: &str) -> f64 {
        text.parse::<Altitude>().unwrap().meters
    }

    #[test]
    fn units_convert_to_meters() {
        assert_eq!(meters("500m"), 500.0);
        assert_eq!(meters("200km"), 200_000.0);
        assert_eq!(meters("1.5Mm"), 1_500_000.0);
        assert_eq!(meters("2Gm"), 2e9);
    }

    #[test]
    fn a_bare_number_is_kilometers() {
        assert_eq!(meters("1.5"), 1_500.0);
    }

    #[test]
    fn an_altitude_may_be_zero_or_below_the_ground() {
        assert_eq!(meters("0m"), 0.0);
        assert_eq!(meters("-25m"), -25.0);
    }

    #[test]
    fn malformed_altitudes_are_rejected() {
        for text in ["", "km", "1mm", "1KM", "infkm", "1 km", "high"] {
            assert!(text.parse::<Altitude>().is_err(), "{text:?}");
        }
    }
}
