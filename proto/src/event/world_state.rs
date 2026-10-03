use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeRate};

use super::{PackReference, Player};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct WorldState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub time: SimulationTime,
    pub rate: TimeRate,
    pub players: Vec<Player>,
}
