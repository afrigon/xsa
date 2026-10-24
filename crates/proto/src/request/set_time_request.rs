use bitcode::{Decode, Encode};
use xsa_units::SimulationTime;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeRequest {
    pub time: SimulationTime,
}

impl RequestType for SetTimeRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::ANY;
}
