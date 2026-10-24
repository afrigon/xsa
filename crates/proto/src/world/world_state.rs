use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow, TimeRate};

use super::PackReference;
use crate::session::SessionEnvironment;
use crate::user::User;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Everything a client needs to replicate the session: sent as a [`Snapshot`](crate::frame::ServerFrame::Snapshot).
pub struct WorldState {
    /// The id of the simulation being run, such as `system-solar:sol`.
    pub simulation: String,
    /// The packs the server loaded; a client loads the same versions.
    pub packs: Vec<PackReference>,
    /// The simulation time when the snapshot was taken.
    pub time: SimulationTime,
    /// How time runs.
    pub time_flow: TimeFlow,
    /// The rate time runs at when the snapshot was taken: the configured one, or a warp's.
    pub rate: TimeRate,
    /// The kind of session, which decides the requests available in it.
    pub environment: SessionEnvironment,
    /// The users in the session.
    pub users: Vec<User>,
}
