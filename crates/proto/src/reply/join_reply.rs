use bitcode::{Decode, Encode};

use crate::user::UserId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// The reply to a [`JoinRequest`](crate::request::JoinRequest).
pub struct JoinReply {
    /// The user the client joined as.
    pub user: UserId,
}
