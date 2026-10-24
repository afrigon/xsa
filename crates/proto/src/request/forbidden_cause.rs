use std::fmt;

use bitcode::{Decode, Encode};

use crate::session::SessionEnvironment;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ForbiddenCause {
    NotJoined,
    Unavailable { environment: SessionEnvironment },
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
