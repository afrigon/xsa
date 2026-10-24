use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::event::Event;
use crate::frame::ClientFrame;

pub struct ClientLink {
    pub requests: UnboundedReceiver<ClientFrame>,
    pub events: UnboundedSender<Event>,
}
