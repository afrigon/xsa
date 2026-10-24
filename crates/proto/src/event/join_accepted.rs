use bitcode::{Decode, Encode};

use crate::world::WorldState;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinAccepted {
    pub state: WorldState,
}
