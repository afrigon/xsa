use std::fmt;

use bitcode::{Decode, Encode};

use super::ForbiddenCause;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Why the server refused a request. Each case says what the sender can do about it; `Display` gives the text to show.
pub enum RequestError {
    /// The request's arguments are invalid; sending different ones may succeed.
    Invalid {
        /// What is wrong with the arguments.
        reason: String,
    },
    /// The request is not open to this sender in this session; sending it again will not succeed.
    Forbidden {
        /// Why the request is not open to the sender.
        cause: ForbiddenCause,
    },
    /// The request is valid and allowed, but the session's current state refuses it.
    Conflict {
        /// What in the session's state refuses the request.
        reason: String,
    },
}

impl RequestError {
    /// An [`Invalid`](RequestError::Invalid) error with the reason to show.
    pub fn invalid(reason: impl Into<String>) -> RequestError {
        RequestError::Invalid { reason: reason.into() }
    }

    /// A [`Conflict`](RequestError::Conflict) error with the reason to show.
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
