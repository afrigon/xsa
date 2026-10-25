use bitcode::{Decode, Encode};
use xsa_units::SimulationDuration;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Advances simulation time by a duration at once, from the time the server holds.
pub struct StepTimeRequest {
    /// How far to advance: finite and greater than 0.
    pub duration: SimulationDuration,
}

impl RequestType for StepTimeRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
