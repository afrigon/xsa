use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId {
    pub value: u64,
}
