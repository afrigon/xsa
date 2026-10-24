use bitcode::{Decode, Encode};

use super::Role;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinRequest {
    pub role: Role,
}
