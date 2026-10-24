mod join_reply;

pub use join_reply::JoinReply;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Reply {
    Empty,
    Join(JoinReply),
}

impl From<()> for Reply {
    fn from(_empty: ()) -> Reply {
        Reply::Empty
    }
}

impl From<JoinReply> for Reply {
    fn from(reply: JoinReply) -> Reply {
        Reply::Join(reply)
    }
}
