use super::DisconnectReason;

#[derive(Debug)]
pub enum Received<Frame> {
    Frame(Frame),
    Disconnected { reason: DisconnectReason },
}
