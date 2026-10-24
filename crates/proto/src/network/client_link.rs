use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{DisconnectReason, Received};
use crate::frame::{ClientFrame, ServerFrame};

pub struct ClientLink {
    pub client_frames: UnboundedReceiver<Received<ClientFrame>>,
    pub server_frames: UnboundedSender<Received<ServerFrame>>,
}

impl ClientLink {
    pub fn close(&self, reason: DisconnectReason) {
        let _ = self.server_frames.send(Received::Disconnected { reason });
    }
}
