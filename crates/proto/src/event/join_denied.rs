use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct JoinDenied {
    pub reason: String,
}
