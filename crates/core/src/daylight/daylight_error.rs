use std::fmt;

use super::TimeOfDay;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DaylightError {
    // The star never reaches the elevation there that day: polar day or polar night.
    NoCrossing { time_of_day: TimeOfDay },
    // The star stands still in the body's sky, which always shows it the same face.
    StarFixed,
    DidNotConverge { time_of_day: TimeOfDay },
}

impl fmt::Display for DaylightError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DaylightError::NoCrossing { time_of_day } => {
                write!(
                    formatter,
                    "there is no {time_of_day} at this latitude today (polar day or night)"
                )
            }
            DaylightError::StarFixed => formatter.write_str("the star stands still in this body's sky"),
            DaylightError::DidNotConverge { time_of_day } => write!(formatter, "could not find the {time_of_day}"),
        }
    }
}

impl std::error::Error for DaylightError {}
