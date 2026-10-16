use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct TimeChanged {
    pub time: SimulationTime,
    pub time_flow: TimeFlow,
}
