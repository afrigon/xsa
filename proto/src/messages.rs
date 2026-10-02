use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId {
    pub value: u64,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct ClientFrame {
    pub id: MessageId,
    pub message: ClientMessage,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Join { role: Role },
    Leave,
    SetTimeRate { rate: f64 },
    SetTime { time: f64 },
    StepTime { seconds: f64 },
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Role {
    Player { name: String },
    Console,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ServerEvent {
    JoinAccepted { state: WorldState },
    JoinDenied { reason: String },
    PlayerJoined { player: Player },
    PlayerLeft { player: PlayerId },
    TimeChanged { time: f64, rate: f64 },
    Tick { time: f64 },
    Reply { id: MessageId, outcome: Outcome },
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Outcome {
    Accepted,
    Denied { reason: String },
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct WorldState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub time: f64,
    pub rate: f64,
    pub players: Vec<Player>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId {
    pub value: u32,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct PackReference {
    pub id: String,
    pub version: String,
}
