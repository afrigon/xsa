use anyhow::{anyhow, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::messages::{ClientFrame, ServerEvent};

pub struct Connection {
    pub(crate) messages: UnboundedSender<ClientFrame>,
    pub(crate) events: UnboundedReceiver<ServerEvent>,
}

pub struct ClientLink {
    pub messages: UnboundedReceiver<ClientFrame>,
    pub events: UnboundedSender<ServerEvent>,
}

impl Connection {
    pub fn local() -> (Connection, ClientLink) {
        let (message_sender, message_receiver) = unbounded_channel();
        let (event_sender, event_receiver) = unbounded_channel();
        let connection = Connection {
            messages: message_sender,
            events: event_receiver,
        };
        let link = ClientLink {
            messages: message_receiver,
            events: event_sender,
        };
        (connection, link)
    }

    pub fn send(&self, frame: ClientFrame) -> anyhow::Result<()> {
        self.messages
            .send(frame)
            .map_err(|_| anyhow!("the server has disconnected"))
    }

    pub async fn receive(&mut self) -> anyhow::Result<ServerEvent> {
        self.events
            .recv()
            .await
            .ok_or_else(|| anyhow!("the server has disconnected"))
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<ServerEvent>> {
        match self.events.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => bail!("the server has disconnected"),
        }
    }
}
