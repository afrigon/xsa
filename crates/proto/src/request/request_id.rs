use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Identifies a request within one connection; its reply carries the same id.
pub struct RequestId {
    /// Assigned by the sender, unique among the requests it sent on this connection.
    pub value: u64,
}
