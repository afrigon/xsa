mod session_environment;
mod session_environments;
mod session_state;

pub use session_environment::SessionEnvironment;
pub use session_environments::SessionEnvironments;
pub use session_state::SessionState;

use anyhow::{Context, bail};

use crate::frame::{ClientFrame, ServerFrame};
use crate::network::{Connection, DisconnectReason};
use crate::reply::Reply;
use crate::request::{JoinRequest, Request, RequestId};
use crate::user::Actor;

pub struct ServerSession {
    connection: Connection,
    next_id: u64,
    join: Option<RequestId>,
    actor: Option<Actor>,
    state: Option<SessionState>,
}

impl ServerSession {
    pub fn join(connection: Connection) -> anyhow::Result<ServerSession> {
        let mut session = ServerSession::new(connection, None);
        let id = session.send(Request::Join(JoinRequest))?;
        session.join = Some(id);

        Ok(session)
    }

    pub fn operator(connection: Connection) -> ServerSession {
        ServerSession::new(connection, Some(Actor::Operator))
    }

    fn new(connection: Connection, actor: Option<Actor>) -> ServerSession {
        ServerSession {
            connection,
            next_id: 0,
            join: None,
            actor,
            state: None,
        }
    }

    // The session is established once both the snapshot and the actor it acts as have arrived.
    pub fn state(&self) -> Option<&SessionState> {
        self.actor.and(self.state.as_ref())
    }

    pub fn send(&mut self, request: Request) -> anyhow::Result<RequestId> {
        let id = RequestId { value: self.next_id };
        self.next_id += 1;
        self.connection.send(ClientFrame { id, request })?;

        Ok(id)
    }

    pub fn poll(&mut self) -> anyhow::Result<Option<ServerFrame>> {
        let frame = self.connection.poll().context("disconnected from the server")?;

        if let Some(frame) = &frame {
            self.apply(frame)?;
        }

        Ok(frame)
    }

    pub fn close(&mut self, reason: DisconnectReason) {
        self.connection.close(reason);
    }

    pub async fn receive(&mut self) -> anyhow::Result<ServerFrame> {
        let frame = self
            .connection
            .receive()
            .await
            .context("disconnected from the server")?;
        self.apply(&frame)?;

        Ok(frame)
    }

    fn apply(&mut self, frame: &ServerFrame) -> anyhow::Result<()> {
        match frame {
            ServerFrame::Snapshot(state) => self.state = Some(SessionState::new(state)),
            ServerFrame::Event(event) => {
                if let Some(state) = &mut self.state {
                    state.apply(event);
                }
            }
            ServerFrame::Reply { id, result } if self.join == Some(*id) => {
                self.join = None;

                match result {
                    Ok(Reply::Join(reply)) => self.actor = Some(Actor::User(reply.user)),
                    Ok(reply) => bail!("the server answered the join with {reply:?}"),
                    Err(error) => bail!("the server refused to let us join: {error}"),
                }
            }
            ServerFrame::Reply { .. } => {}
        }

        Ok(())
    }
}
