use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeRateOverrideRequest {
    pub rate: Option<TimeRate>,
}

impl SetTimeRateOverrideRequest {
    pub const SESSION_ENVIRONMENT: SessionEnvironments = SessionEnvironments::SINGLEPLAYER;
}
