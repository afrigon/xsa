use bitcode::{Decode, Encode};

use super::PlayerId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct PlayerLeft {
    pub player: PlayerId,
}
