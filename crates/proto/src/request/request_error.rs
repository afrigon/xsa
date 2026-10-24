use std::fmt;

use bitcode::{Decode, Encode};

use super::ForbiddenCause;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum RequestError {
    Invalid { reason: String },
    Forbidden { cause: ForbiddenCause },
    Conflict { reason: String },
}

impl RequestError {
    pub fn invalid(reason: impl Into<String>) -> RequestError {
        RequestError::Invalid { reason: reason.into() }
    }

    pub fn conflict(reason: impl Into<String>) -> RequestError {
        RequestError::Conflict { reason: reason.into() }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::Invalid { reason } | RequestError::Conflict { reason } => write!(formatter, "{reason}"),
            RequestError::Forbidden { cause } => write!(formatter, "{cause}"),
        }
    }
}
