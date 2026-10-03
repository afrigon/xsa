use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct PackReference {
    pub id: String,
    pub version: String,
}
