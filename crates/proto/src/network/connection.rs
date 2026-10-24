use anyhow::{anyhow, bail};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::{ClientLink, LocalConnection, RemoteConnection};
use crate::frame::{ClientFrame, ServerFrame};

pub struct Connection {
    pub(crate) client_frames: UnboundedSender<ClientFrame>,
    pub(crate) server_frames: UnboundedReceiver<ServerFrame>,
}

impl Connection {
    pub fn local() -> LocalConnection {
        let (client_frame_sender, client_frame_receiver) = unbounded_channel();
        let (server_frame_sender, server_frame_receiver) = unbounded_channel();

        LocalConnection {
            connection: Connection {
                client_frames: client_frame_sender,
                server_frames: server_frame_receiver,
            },
            link: ClientLink {
                client_frames: client_frame_receiver,
                server_frames: server_frame_sender,
            },
        }
    }

    pub fn remote(address: &str, fingerprint: &str) -> anyhow::Result<Connection> {
        RemoteConnection::start(address, fingerprint.parse()?)
    }

    pub fn send(&self, frame: ClientFrame) -> anyhow::Result<()> {
        self.client_frames
            .send(frame)
            .map_err(|_| anyhow!("the server has disconnected"))
    }

    pub async fn receive(&mut self) -> anyhow::Result<ServerFrame> {
        self.server_frames
            .recv()
            .await
            .ok_or_else(|| anyhow!("the server has disconnected"))
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<ServerFrame>> {
        match self.server_frames.try_recv() {
            Ok(frame) => Ok(Some(frame)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => bail!("the server has disconnected"),
        }
    }
}
