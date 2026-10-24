use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId {
    pub value: u64,
}
