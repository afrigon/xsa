use std::str::FromStr;

use xsa_units::SimulationTime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timestamp {
    pub time: SimulationTime,
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let time = text.parse().map_err(|err| format!("{err:#}"))?;

        Ok(Timestamp { time })
    }
}
