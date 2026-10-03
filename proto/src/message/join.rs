use bitcode::{Decode, Encode};

use super::Role;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Join {
    pub role: Role,
}
