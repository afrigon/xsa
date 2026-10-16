use std::fmt;

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEnvironment {
    Singleplayer,
    LanHost,
    Dedicated,
}

impl fmt::Display for SessionEnvironment {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            SessionEnvironment::Singleplayer => "singleplayer",
            SessionEnvironment::LanHost => "LAN host",
            SessionEnvironment::Dedicated => "dedicated server",
        };

        write!(formatter, "{name}")
    }
}
