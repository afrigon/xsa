use std::time::Instant;

use crate::connection::Connection;
use crate::messages::{ClientFrame, ClientMessage, MessageId, PackReference, Player, ServerEvent, WorldState};

pub struct ServerSession {
    connection: Connection,
    next_id: u64,
    state: Option<SessionState>,
}

pub struct SessionState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub players: Vec<Player>,
    time: f64,
    rate: f64,
    time_received: Instant,
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
        match event {
            ServerEvent::JoinAccepted { state } => self.state = Some(SessionState::new(state)),
            ServerEvent::PlayerJoined { player } => {
                if let Some(state) = &mut self.state {
                    state.players.push(player.clone());
                }
            }
            ServerEvent::PlayerLeft { player } => {
                if let Some(state) = &mut self.state {
                    state.players.retain(|joined| joined.id != *player);
                }
            }
            ServerEvent::TimeChanged { time, rate } => {
                if let Some(state) = &mut self.state {
                    state.set_time(*time);
                    state.rate = *rate;
                }
            }
            ServerEvent::Tick { time } => {
                if let Some(state) = &mut self.state {
                    state.set_time(*time);
                }
            }
            ServerEvent::JoinDenied { .. } | ServerEvent::Reply { .. } => {}
        }
    }
}

impl SessionState {
    fn new(state: &WorldState) -> Self {
        Self {
            simulation: state.simulation.clone(),
            packs: state.packs.clone(),
            players: state.players.clone(),
            time: state.time,
            rate: state.rate,
            time_received: Instant::now(),
        }
    }

    pub fn time(&self) -> f64 {
        self.time + self.time_received.elapsed().as_secs_f64() * self.rate
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    fn set_time(&mut self, time: f64) {
        self.time = time;
        self.time_received = Instant::now();
    }
}
