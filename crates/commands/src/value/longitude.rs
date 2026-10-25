use std::str::FromStr;

use xsa_core::daylight::TimeOfDay;

// Where `camera tp` stands east or west: degrees, a time of day at the latitude, or the meridian under the parent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Longitude {
    Degrees { degrees: f64 },
    TimeOfDay { time_of_day: TimeOfDay },
    // Where the body's parent is overhead.
    NearSide,
    FarSide,
}

impl Longitude {
    pub fn names() -> impl Iterator<Item = &'static str> {
        TimeOfDay::ALL
            .into_iter()
            .map(TimeOfDay::name)
            .chain(["near-side", "far-side"])
    }
}

impl FromStr for Longitude {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Ok(degrees) = text.parse::<f64>()
            && degrees.is_finite()
        {
            return Ok(Longitude::Degrees { degrees });
        }

        match text {
            "near-side" => Ok(Longitude::NearSide),
            "far-side" => Ok(Longitude::FarSide),
            _ => text
                .parse()
                .map(|time_of_day| Longitude::TimeOfDay { time_of_day })
                .map_err(|_| {
                    let names: Vec<&str> = Longitude::names().collect();
                    format!("{text:?} is not a longitude: degrees or one of {}", names.join(", "))
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_degrees_and_words_are_names() {
        assert_eq!("120".parse(), Ok(Longitude::Degrees { degrees: 120.0 }));
        assert_eq!(
            "sunset".parse(),
            Ok(Longitude::TimeOfDay {
                time_of_day: TimeOfDay::Sunset
            })
        );
        assert_eq!("near-side".parse(), Ok(Longitude::NearSide));
        assert!("east".parse::<Longitude>().is_err());
    }
}
