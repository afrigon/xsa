use bitcode::{Decode, Encode};

use super::PlayerId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
}
