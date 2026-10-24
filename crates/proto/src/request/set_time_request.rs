use bitcode::{Decode, Encode};
use xsa_units::SimulationTime;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Jumps simulation time to a date.
pub struct SetTimeRequest {
    /// The new simulation time: finite.
    pub time: SimulationTime,
}

impl RequestType for SetTimeRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
