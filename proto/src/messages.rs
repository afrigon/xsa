use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Join,
    Leave,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ServerEvent {
    Joined {
        simulation: String,
        packs: Vec<PackReference>,
        time: f64,
    },
    JoinDenied {
        reason: String,
    },
    Tick {
        time: f64,
    },
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct PackReference {
    pub id: String,
    pub version: String,
}
