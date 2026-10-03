use bitcode::{Decode, Encode};
use xsa_units::SimulationTime;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTime {
    pub time: SimulationTime,
}
