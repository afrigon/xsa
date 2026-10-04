mod session_state;

pub use session_state::SessionState;

use crate::connection::Connection;
use crate::event::ServerEvent;
use crate::message::{ClientFrame, ClientMessage, MessageId};

pub struct ServerSession {
    connection: Connection,
    next_id: u64,
    state: Option<SessionState>,
}

impl ServerSession {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            next_id: 0,
            state: None,
        }
    }

    pub fn state(&self) -> Option<&SessionState> {
        self.state.as_ref()
    }

    pub fn send(&mut self, message: ClientMessage) -> anyhow::Result<MessageId> {
        let id = MessageId { value: self.next_id };
        self.next_id += 1;
        self.connection.send(ClientFrame { id, message })?;

        Ok(id)
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<ServerEvent>> {
        let event = self.connection.poll()?;

        if let Some(event) = &event {
            self.apply(event);
        }

        Ok(event)
    }

    pub async fn receive(&mut self) -> anyhow::Result<ServerEvent> {
        let event = self.connection.receive().await?;
        self.apply(&event);

        Ok(event)
    }

    fn apply(&mut self, event: &ServerEvent) {
        if let ServerEvent::JoinAccepted(accepted) = event {
            self.state = Some(SessionState::new(&accepted.state));
        } else if let Some(state) = &mut self.state {
            state.apply(event);
        }
    }
}
