use std::fmt;

use bitcode::{Decode, Encode};

use crate::session::SessionEnvironment;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Why a request was [`Forbidden`](super::RequestError::Forbidden): it is not open to this sender in this session.
pub enum ForbiddenCause {
    /// The sender has not joined the session, and only a [`JoinRequest`](super::JoinRequest) is open to it.
    NotJoined,
    /// The request is not available in this kind of session ([`RequestType::SESSION_ENVIRONMENTS`](super::RequestType::SESSION_ENVIRONMENTS)).
    Unavailable {
        /// The kind of session the request was sent to.
        environment: SessionEnvironment,
    },
}

impl fmt::Display for ForbiddenCause {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ForbiddenCause::NotJoined => write!(formatter, "join the simulation first"),
            ForbiddenCause::Unavailable { environment } => {
                write!(formatter, "not available in this session ({environment})")
            }
        }
    }
}
