use bitcode::{Decode, Encode};

use crate::user::UserId;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// A user left the session, or their connection ended.
pub struct UserLeftEvent {
    /// The user who left.
    pub user: UserId,
}
