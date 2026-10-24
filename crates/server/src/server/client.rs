use tokio::sync::mpsc::error::TryRecvError;
use xsa_proto::frame::{ClientFrame, ServerFrame};
use xsa_proto::network::{ClientLink, DisconnectReason, Received};
use xsa_proto::user::Actor;

pub(super) struct Client {
    pub link: ClientLink,
    pub actor: Option<Actor>,
    pub disconnected: Option<DisconnectReason>,
}

impl Client {
    pub fn new(link: ClientLink, actor: Option<Actor>) -> Client {
        Client {
            link,
            actor,
            disconnected: None,
        }
    }

    pub fn receive(&mut self) -> Option<ClientFrame> {
        if self.disconnected.is_some() {
            return None;
        }

        match self.link.client_frames.try_recv() {
            Ok(Received::Frame(frame)) => Some(frame),
            Ok(Received::Disconnected { reason }) => {
                self.disconnected = Some(reason);
                None
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.disconnected = Some(DisconnectReason::Lost);
                None
            }
        }
    }

    pub fn send(&mut self, frame: ServerFrame) {
        if self.disconnected.is_none() && self.link.server_frames.send(Received::Frame(frame)).is_err() {
            self.disconnected = Some(DisconnectReason::Lost);
        }
    }
}
