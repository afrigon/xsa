use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow};

use super::PackReference;
use crate::session::SessionEnvironment;
use crate::user::User;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct WorldState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub time: SimulationTime,
    pub time_flow: TimeFlow,
    pub environment: SessionEnvironment,
    pub users: Vec<User>,
}
