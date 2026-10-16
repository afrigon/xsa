use std::str::FromStr;

use xsa_units::TimeRate;

const NONE: &str = "none";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeOverride {
    pub rate: Option<TimeRate>,
}

impl FromStr for TimeOverride {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text == NONE {
            return Ok(TimeOverride { rate: None });
        }

        let multiplier = text
            .parse()
            .map_err(|_| format!("expected a time rate or {NONE}, got {text}"))?;

        Ok(TimeOverride {
            rate: Some(TimeRate { multiplier }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_rate_or_none() {
        assert_eq!(
            "0.25".parse(),
            Ok(TimeOverride {
                rate: Some(TimeRate { multiplier: 0.25 })
            })
        );
        assert_eq!("none".parse(), Ok(TimeOverride { rate: None }));
        assert!("fast".parse::<TimeOverride>().is_err());
    }
}
