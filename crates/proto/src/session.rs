mod session_environment;
mod session_environments;
mod session_state;

pub use session_environment::SessionEnvironment;
pub use session_environments::SessionEnvironments;
pub use session_state::SessionState;

use crate::event::Event;
use crate::frame::ClientFrame;
use crate::network::Connection;
use crate::request::{Request, RequestId};

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

    pub fn send(&mut self, request: Request) -> anyhow::Result<RequestId> {
        let id = RequestId { value: self.next_id };
        self.next_id += 1;
        self.connection.send(ClientFrame { id, request })?;

        Ok(id)
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<Event>> {
        let event = self.connection.poll()?;

        if let Some(event) = &event {
            self.apply(event);
        }

        Ok(event)
    }

    pub async fn receive(&mut self) -> anyhow::Result<Event> {
        let event = self.connection.receive().await?;
        self.apply(&event);

        Ok(event)
    }

    fn apply(&mut self, event: &Event) {
        if let Event::JoinAccepted(accepted) = event {
            self.state = Some(SessionState::new(&accepted.state));
        } else if let Some(state) = &mut self.state {
            state.apply(event);
        }
    }
}
