use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Sets the configured rate simulation time runs at.
pub struct SetTimeRateRequest {
    /// The new rate: finite and at least 0.
    pub rate: TimeRate,
}

impl RequestType for SetTimeRateRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
