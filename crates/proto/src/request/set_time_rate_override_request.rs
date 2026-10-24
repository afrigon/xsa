use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Runs time at a rate without changing the configured one, or returns to the configured rate. Singleplayer only:
/// the pause menu stops time with one.
pub struct SetTimeRateOverrideRequest {
    /// The rate time runs at until the override is cleared, or `None` to clear it.
    pub rate: Option<TimeRate>,
}

impl RequestType for SetTimeRateOverrideRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::SINGLEPLAYER;
}
