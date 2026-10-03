use bitcode::{Decode, Encode};

use crate::TICK_RATE_HERTZ;

#[derive(Encode, Decode, Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct SimulationDuration {
    pub seconds: f64,
}

impl SimulationDuration {
    pub fn from_ticks(ticks: u64) -> SimulationDuration {
        SimulationDuration {
            seconds: ticks as f64 / TICK_RATE_HERTZ,
        }
    }

    pub fn is_valid_step(self) -> bool {
        self.seconds.is_finite() && self.seconds > 0.0
    }
}
