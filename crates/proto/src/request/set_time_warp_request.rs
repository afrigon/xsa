use bitcode::{Decode, Encode};
use xsa_units::TimeWarp;

use super::RequestType;
use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Runs time ahead to a moment, then back to the configured rate, or stops a running warp. Singleplayer only. A time
/// rate override pauses the warp until it is cleared; any other change to time stops it.
pub struct SetTimeWarpRequest {
    /// The warp to run, replacing a running one, or `None` to stop the running one where it is.
    pub warp: Option<TimeWarp>,
}

impl RequestType for SetTimeWarpRequest {
    type Reply = ();

    const SESSION_ENVIRONMENTS: SessionEnvironments = SessionEnvironments::SINGLEPLAYER;
}
