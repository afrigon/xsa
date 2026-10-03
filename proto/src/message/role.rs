use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Role {
    Player { name: String },
    Server,
}
