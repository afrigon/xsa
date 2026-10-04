use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeRate};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct TimeChanged {
    pub time: SimulationTime,
    pub rate: TimeRate,
}
