use bitcode::{Decode, Encode};
use xsa_units::TimeRate;

use crate::session::SessionEnvironments;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SetTimeOverride {
    pub rate: Option<TimeRate>,
}

impl SetTimeOverride {
    pub const SESSION_ENVIRONMENT: SessionEnvironments = SessionEnvironments::SINGLEPLAYER;
}
