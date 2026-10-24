mod actor;
mod user_id;

pub use actor::Actor;
pub use user_id::UserId;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct User {
    pub id: UserId,
}
