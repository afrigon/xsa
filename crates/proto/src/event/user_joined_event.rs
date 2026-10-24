use bitcode::{Decode, Encode};

use crate::user::User;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// A user joined the session. Sent to every member, the joining user included.
pub struct UserJoinedEvent {
    /// The user who joined.
    pub user: User,
}
