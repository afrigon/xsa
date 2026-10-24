//! Who takes part in a session.

mod actor;
mod user_id;

pub use actor::Actor;
pub use user_id::UserId;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// A person in the session.
pub struct User {
    /// Identifies the user for as long as they stay in the session.
    pub id: UserId,
}
