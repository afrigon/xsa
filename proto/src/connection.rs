mod client_link;
mod local_connection;

pub use client_link::ClientLink;
pub use local_connection::LocalConnection;

use anyhow::{anyhow, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::event::ServerEvent;
use crate::message::ClientFrame;
use crate::network::RemoteConnection;

pub struct Connection {
    pub(crate) messages: UnboundedSender<ClientFrame>,
    pub(crate) events: UnboundedReceiver<ServerEvent>,
}

impl Connection {
    pub fn local() -> LocalConnection {
        let (message_sender, message_receiver) = unbounded_channel();
        let (event_sender, event_receiver) = unbounded_channel();

        LocalConnection {
            connection: Connection {
                messages: message_sender,
                events: event_receiver,
            },
            link: ClientLink {
                messages: message_receiver,
                events: event_sender,
            },
        }
    }

    pub fn remote(address: &str, fingerprint: &str) -> anyhow::Result<Connection> {
        RemoteConnection::start(address, fingerprint.parse()?)
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
