use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::event::ServerEvent;
use crate::message::ClientFrame;

pub struct ClientLink {
    pub messages: UnboundedReceiver<ClientFrame>,
    pub events: UnboundedSender<ServerEvent>,
}
