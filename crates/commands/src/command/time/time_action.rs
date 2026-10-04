use usage::Subcommands;

use super::{TimePauseCommand, TimeRateCommand, TimeResumeCommand, TimeSetCommand, TimeStepCommand};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum TimeAction {
    /// Jump to a UTC date and time, e.g. 2026-10-02T12:00:00Z, or now
    Set(TimeSetCommand),
    /// Run the simulation at a multiple of real time; 0 pauses
    Rate(TimeRateCommand),
    /// Stop time (time rate 0)
    Pause(TimePauseCommand),
    /// Run time at real speed (time rate 1)
    Resume(TimeResumeCommand),
    /// Advance time by a duration: s, min, h, or t for ticks
    Step(TimeStepCommand),
}

impl Routable for TimeAction {
    fn route(self) -> Route {
        match self {
            TimeAction::Set(command) => command.route(),
            TimeAction::Rate(command) => command.route(),
            TimeAction::Pause(command) => command.route(),
            TimeAction::Resume(command) => command.route(),
            TimeAction::Step(command) => command.route(),
        }
    }
}
