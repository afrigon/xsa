use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow};

use super::{PackReference, Player};
use crate::session::SessionEnvironment;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct WorldState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub time: SimulationTime,
    pub time_flow: TimeFlow,
    pub environment: SessionEnvironment,
    pub players: Vec<Player>,
}
