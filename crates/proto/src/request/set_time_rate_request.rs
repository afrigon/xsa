use bitcode::{Decode, Encode};
use xsa_units::TimeRateChange;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeRateRequest {
    pub change: TimeRateChange,
}
