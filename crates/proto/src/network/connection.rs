use anyhow::anyhow;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::{ClientLink, DisconnectReason, LocalConnection, Received, RemoteConnection};
use crate::frame::{ClientFrame, ServerFrame};

pub struct Connection {
    pub(crate) client_frames: UnboundedSender<Received<ClientFrame>>,
    pub(crate) server_frames: UnboundedReceiver<Received<ServerFrame>>,
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
            .send(Received::Frame(frame))
            .map_err(|_| anyhow!("the server has disconnected"))
    }

    pub async fn receive(&mut self) -> Result<ServerFrame, DisconnectReason> {
        Connection::frame_or_reason(self.server_frames.recv().await)
    }

    pub fn poll(&mut self) -> Result<Option<ServerFrame>, DisconnectReason> {
        match self.server_frames.try_recv() {
            Ok(received) => Connection::frame_or_reason(Some(received)).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(DisconnectReason::Lost),
        }
    }

    // Blocks until the other end releases the link, so the reason reaches the server before the process exits.
    pub fn close(&mut self, reason: DisconnectReason) {
        let _ = self.client_frames.send(Received::Disconnected { reason });

        while self.server_frames.blocking_recv().is_some() {}
    }

    fn frame_or_reason(received: Option<Received<ServerFrame>>) -> Result<ServerFrame, DisconnectReason> {
        match received {
            Some(Received::Frame(frame)) => Ok(frame),
            Some(Received::Disconnected { reason }) => Err(reason),
            None => Err(DisconnectReason::Lost),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn closing_delivers_the_reason_and_waits_for_the_link_to_be_released() {
        let LocalConnection {
            mut connection,
            mut link,
        } = Connection::local();
        let closing = thread::spawn(move || connection.close(DisconnectReason::Left));

        let Some(Received::Disconnected { reason }) = link.client_frames.blocking_recv() else {
            panic!("the reason was not delivered");
        };
        assert_eq!(reason, DisconnectReason::Left);
        assert!(!closing.is_finished());

        drop(link);
        closing.join().unwrap();
    }

    #[test]
    fn a_dropped_link_is_lost() {
        let LocalConnection { mut connection, link } = Connection::local();
        drop(link);
        assert_eq!(connection.poll().unwrap_err(), DisconnectReason::Lost);
    }
}
