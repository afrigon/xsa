use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// A pack at a version.
pub struct PackReference {
    /// The pack's id.
    pub id: String,
    /// The pack's version.
    pub version: String,
}
