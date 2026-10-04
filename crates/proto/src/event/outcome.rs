use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Outcome {
    Accepted,
    Denied { reason: String },
}

impl Outcome {
    pub fn denied(reason: impl Into<String>) -> Outcome {
        Outcome::Denied { reason: reason.into() }
    }
}
