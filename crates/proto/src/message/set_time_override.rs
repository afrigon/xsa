use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeOverride {
    pub rate: Option<TimeRate>,
}
