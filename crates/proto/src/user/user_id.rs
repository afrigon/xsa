use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Identifies a user within one session.
pub struct UserId {
    /// Assigned by the server, never reused within the session.
    pub value: u32,
}
