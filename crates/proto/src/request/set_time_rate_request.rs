use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeRateRequest {
    pub rate: TimeRate,
}

impl RequestType for SetTimeRateRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
