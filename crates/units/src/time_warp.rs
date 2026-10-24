use bitcode::{Decode, Encode};

use crate::{SimulationTime, WarpPace};

// Time running ahead to a moment, then back to its rate.
#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq)]
pub struct TimeWarp {
    pub until: SimulationTime,
    pub pace: WarpPace,
}
