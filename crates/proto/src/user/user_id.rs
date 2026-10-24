use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId {
    pub value: u32,
}
