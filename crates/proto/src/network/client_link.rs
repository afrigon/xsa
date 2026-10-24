use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::frame::{ClientFrame, ServerFrame};

pub struct ClientLink {
    pub client_frames: UnboundedReceiver<ClientFrame>,
    pub server_frames: UnboundedSender<ServerFrame>,
}
