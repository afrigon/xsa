use bitcode::{Decode, Encode};

use crate::user::User;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct UserJoinedEvent {
    pub user: User,
}
