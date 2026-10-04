use bitcode::{Decode, Encode};

use super::Player;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct PlayerJoined {
    pub player: Player,
}
