use std::time::Instant;

use xsa_units::{SimulationDuration, SimulationTime, TimeRate};

use crate::event::{PackReference, Player, ServerEvent, WorldState};

pub struct SessionState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub players: Vec<Player>,
    time: SimulationTime,
    rate: TimeRate,
    time_received: Instant,
}

impl SessionState {
    pub(super) fn new(state: &WorldState) -> Self {
        Self {
            simulation: state.simulation.clone(),
            packs: state.packs.clone(),
            players: state.players.clone(),
            time: state.time,
            rate: state.rate,
            time_received: Instant::now(),
        }
    }

    pub fn time(&self) -> SimulationTime {
        let elapsed = SimulationDuration {
            seconds: self.time_received.elapsed().as_secs_f64(),
        };

        self.time.advanced_by(self.rate.scale(elapsed))
    }

    pub fn rate(&self) -> TimeRate {
        self.rate
    }

    pub(super) fn apply(&mut self, event: &ServerEvent) {
        match event {
            ServerEvent::PlayerJoined(joined) => self.players.push(joined.player.clone()),
            ServerEvent::PlayerLeft(left) => self.players.retain(|player| player.id != left.player),
            ServerEvent::TimeChanged(changed) => {
                self.set_time(changed.time);
                self.rate = changed.rate;
            }
            ServerEvent::Tick(tick) => self.set_time(tick.time),
            ServerEvent::JoinAccepted(_) | ServerEvent::JoinDenied(_) | ServerEvent::Reply(_) => {}
        }
    }

    fn set_time(&mut self, time: SimulationTime) {
        self.time = time;
        self.time_received = Instant::now();
    }
}
