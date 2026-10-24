use std::time::Instant;

use xsa_units::{SimulationDuration, SimulationTime, TimeFlow};

use super::SessionEnvironment;
use crate::event::Event;
use crate::user::User;
use crate::world::{PackReference, WorldState};

pub struct SessionState {
    pub simulation: String,
    pub packs: Vec<PackReference>,
    pub users: Vec<User>,
    time: SimulationTime,
    time_flow: TimeFlow,
    environment: SessionEnvironment,
    time_received: Instant,
}

impl SessionState {
    pub(super) fn new(state: &WorldState) -> Self {
        Self {
            simulation: state.simulation.clone(),
            packs: state.packs.clone(),
            users: state.users.clone(),
            time: state.time,
            time_flow: state.time_flow,
            environment: state.environment,
            time_received: Instant::now(),
        }
    }

    pub fn time(&self) -> SimulationTime {
        let elapsed = SimulationDuration {
            seconds: self.time_received.elapsed().as_secs_f64(),
        };

        self.time.advanced_by(self.time_flow.effective().scale(elapsed))
    }

    pub fn time_flow(&self) -> TimeFlow {
        self.time_flow
    }

    pub fn environment(&self) -> SessionEnvironment {
        self.environment
    }

    pub(super) fn apply(&mut self, event: &Event) {
        match event {
            Event::UserJoined(joined) => self.users.push(joined.user.clone()),
            Event::UserLeft(left) => self.users.retain(|user| user.id != left.user),
            Event::TimeChanged(changed) => {
                self.set_time(changed.time);
                self.time_flow = changed.time_flow;
            }
            Event::Tick(tick) => self.set_time(tick.time),
        }
    }

    fn set_time(&mut self, time: SimulationTime) {
        self.time = time;
        self.time_received = Instant::now();
    }
}
