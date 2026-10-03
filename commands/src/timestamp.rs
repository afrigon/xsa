use std::str::FromStr;

use xsa_core::time;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timestamp {
    pub time: f64,
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let time = time::parse_timestamp(text).map_err(|err| format!("{err:#}"))?;
        Ok(Timestamp { time })
    }
}
