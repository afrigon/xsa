use bitcode::{Decode, Encode};
use xsa_units::TimeRateStep;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Steps the configured rate one notch faster or slower, from the rate the server holds.
pub struct StepTimeRateRequest {
    /// Which way to step.
    pub step: TimeRateStep,
}

impl RequestType for StepTimeRateRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
