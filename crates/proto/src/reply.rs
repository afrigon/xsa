//! Replies: what the server answers an accepted request with.

mod join_reply;

pub use join_reply::JoinReply;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// What the server answers an accepted request with: one case per reply type a [`RequestType`](crate::request::RequestType) declares.
pub enum Reply {
    /// The reply of requests that return nothing (`()`).
    Empty,
    /// The reply of a [`JoinRequest`](crate::request::JoinRequest).
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
