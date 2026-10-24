use bitcode::{Decode, Encode};
use xsa_units::SimulationDuration;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct StepTimeRequest {
    pub duration: SimulationDuration,
}

impl RequestType for StepTimeRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
