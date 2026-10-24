use bitcode::{Decode, Encode};
use xsa_units::TimeRateStep;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct StepTimeRateRequest {
    pub step: TimeRateStep,
}

impl RequestType for StepTimeRateRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
