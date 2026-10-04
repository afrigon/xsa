use bitcode::{Decode, Encode};

use super::WorldState;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinAccepted {
    pub state: WorldState,
}
