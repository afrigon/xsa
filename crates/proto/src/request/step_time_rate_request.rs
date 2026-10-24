use bitcode::{Decode, Encode};
use xsa_units::TimeRateStep;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct StepTimeRateRequest {
    pub step: TimeRateStep,
}
