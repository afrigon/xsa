use bitcode::{Decode, Encode};

use crate::user::UserId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinReply {
    pub user: UserId,
}
