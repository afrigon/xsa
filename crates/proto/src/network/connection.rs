use anyhow::{anyhow, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::{ClientLink, LocalConnection, RemoteConnection};
use crate::event::Event;
use crate::frame::ClientFrame;

pub struct Connection {
    pub(crate) requests: UnboundedSender<ClientFrame>,
    pub(crate) events: UnboundedReceiver<Event>,
}

impl Connection {
    pub fn local() -> LocalConnection {
        let (request_sender, request_receiver) = unbounded_channel();
        let (event_sender, event_receiver) = unbounded_channel();

        LocalConnection {
            connection: Connection {
                requests: request_sender,
                events: event_receiver,
            },
            link: ClientLink {
                requests: request_receiver,
                events: event_sender,
            },
        }
    }

    pub fn remote(address: &str, fingerprint: &str) -> anyhow::Result<Connection> {
        RemoteConnection::start(address, fingerprint.parse()?)
    }

    pub fn send(&self, frame: ClientFrame) -> anyhow::Result<()> {
        self.requests
            .send(frame)
            .map_err(|_| anyhow!("the server has disconnected"))
    }

    pub async fn receive(&mut self) -> anyhow::Result<Event> {
        self.events
            .recv()
            .await
            .ok_or_else(|| anyhow!("the server has disconnected"))
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<Event>> {
        match self.events.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => bail!("the server has disconnected"),
        }
    }
}
