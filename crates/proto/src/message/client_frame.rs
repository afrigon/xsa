use bitcode::{Decode, Encode};

use super::{ClientMessage, MessageId};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct ClientFrame {
    pub id: MessageId,
    pub message: ClientMessage,
}
