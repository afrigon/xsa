use std::str::FromStr;

use xsa_units::SimulationTime;

const NOW: &str = "now";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timestamp {
    pub time: SimulationTime,
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let time = if text == NOW {
            SimulationTime::now()
        } else {
            text.parse()
        }
        .map_err(|err| format!("{err:#}"))?;

        Ok(Timestamp { time })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_the_current_date() {
        let before = SimulationTime::now().unwrap();
        let parsed: Timestamp = "now".parse().unwrap();
        assert!(parsed.time.seconds_since(before) >= 0.0);
    }
}
