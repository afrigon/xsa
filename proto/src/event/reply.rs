use bitcode::{Decode, Encode};

use super::Outcome;
use crate::message::MessageId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Reply {
    pub id: MessageId,
    pub outcome: Outcome,
}
