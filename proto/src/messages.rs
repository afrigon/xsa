use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Join,
    Leave,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum ServerEvent {
    Welcome { bodies: Vec<BodyDefinition> },
    Tick { time: f64 },
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct BodyDefinition {
    pub id: String,
    pub position: [f64; 3],
    pub orientation: [f64; 4],
    pub radius: f64,
}
