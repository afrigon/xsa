use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::Received;
use crate::frame::{ClientFrame, ServerFrame};

pub struct ClientLink {
    pub client_frames: UnboundedReceiver<Received<ClientFrame>>,
    pub server_frames: UnboundedSender<Received<ServerFrame>>,
}
