mod user_id;

pub use user_id::UserId;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct User {
    pub id: UserId,
    pub name: String,
}
