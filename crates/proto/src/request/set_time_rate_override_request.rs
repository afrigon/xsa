use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeRateOverrideRequest {
    pub rate: Option<TimeRate>,
}

impl RequestType for SetTimeRateOverrideRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::SINGLEPLAYER;
}
