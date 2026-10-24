use bitcode::{Decode, Encode};
use xsa_units::SimulationDuration;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct StepTimeRequest {
    pub duration: SimulationDuration,
}
