use std::str::FromStr;

const METERS_PER_KILOMETER: f64 = 1_000.0;

struct LengthUnit {
    suffix: &'static str,
    meters: f64,
}

// Longest suffixes first: every unit ends with "m".
const LENGTH_UNITS: [LengthUnit; 4] = [
    LengthUnit {
        suffix: "Gm",
        meters: 1e9,
    },
    LengthUnit {
        suffix: "Mm",
        meters: 1e6,
    },
    LengthUnit {
        suffix: "km",
        meters: METERS_PER_KILOMETER,
    },
    LengthUnit {
        suffix: "m",
        meters: 1.0,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Distance {
    pub meters: f64,
}

impl FromStr for Distance {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("{text:?} is not a distance like 200km (units: m, km, Mm, Gm; a bare number is km)");
        let unit = LENGTH_UNITS.iter().find(|unit| text.ends_with(unit.suffix));
        let number = unit.map_or(text, |unit| &text[..text.len() - unit.suffix.len()]);
        let scale = unit.map_or(METERS_PER_KILOMETER, |unit| unit.meters);
        let value: f64 = number.parse().map_err(|_| invalid())?;

        if !(value.is_finite() && value > 0.0) {
            return Err(invalid());
        }

        Ok(Distance { meters: value * scale })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meters(text: &str) -> f64 {
        text.parse::<Distance>().unwrap().meters
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
        assert_eq!(meters("200"), 200_000.0);
    }

    #[test]
    fn malformed_distances_are_rejected() {
        for text in ["", "km", "1mm", "1KM", "-5km", "0", "infkm", "1 km"] {
            assert!(text.parse::<Distance>().is_err(), "{text:?}");
        }
    }
}
