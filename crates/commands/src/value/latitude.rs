use std::str::FromStr;

// Where `camera tp` stands north or south: degrees, or the latitude under another body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Latitude {
    Degrees { degrees: f64 },
    // Where the star is overhead.
    Substellar,
    // Where the star is underfoot, on the far side of the body.
    Antistellar,
    // Where the body's parent is overhead.
    NearSide,
    FarSide,
}

impl Latitude {
    pub const NAMED: [Latitude; 4] = [
        Latitude::Substellar,
        Latitude::Antistellar,
        Latitude::NearSide,
        Latitude::FarSide,
    ];

    pub fn name(self) -> Option<&'static str> {
        match self {
            Latitude::Degrees { .. } => None,
            Latitude::Substellar => Some("substellar"),
            Latitude::Antistellar => Some("antistellar"),
            Latitude::NearSide => Some("near-side"),
            Latitude::FarSide => Some("far-side"),
        }
    }
}

impl FromStr for Latitude {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Ok(degrees) = text.parse::<f64>()
            && degrees.is_finite()
        {
            return Ok(Latitude::Degrees { degrees });
        }

        Latitude::NAMED
            .into_iter()
            .find(|latitude| latitude.name() == Some(text))
            .ok_or_else(|| {
                format!("{text:?} is not a latitude: degrees, substellar, antistellar, near-side or far-side")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_degrees_and_words_are_names() {
        assert_eq!("-12.5".parse(), Ok(Latitude::Degrees { degrees: -12.5 }));
        assert_eq!("substellar".parse(), Ok(Latitude::Substellar));
        assert_eq!("far-side".parse(), Ok(Latitude::FarSide));
        assert!("north".parse::<Latitude>().is_err());
        assert!("inf".parse::<Latitude>().is_err());
    }
}
