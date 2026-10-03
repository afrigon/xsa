use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId {
    pub value: u32,
}
