use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub value: String,
    pub description: Option<String>,
}
