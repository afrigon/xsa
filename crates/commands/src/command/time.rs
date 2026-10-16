mod faster;
mod pause;
mod rate;
mod resume;
mod set;
mod slower;
mod step;
mod time_action;
mod time_override;

pub use faster::TimeFasterCommand;
pub use pause::TimePauseCommand;
pub use rate::TimeRateCommand;
pub use resume::TimeResumeCommand;
pub use set::TimeSetCommand;
pub use slower::TimeSlowerCommand;
pub use step::TimeStepCommand;
pub use time_action::TimeAction;
pub use time_override::TimeOverrideCommand;

use usage::Args;
use xsa_proto::session::SessionState;

use super::{Routable, Route, SessionCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimeCommand {
    #[usage(subcommand)]
    pub action: Option<TimeAction>,
}

impl TimeCommand {
    pub fn describe(state: Option<&SessionState>) -> Output {
        let Some(state) = state else {
            return Output::failure("not joined to a simulation yet");
        };

        Output::success(format!("time: {}, {}", state.time(), state.time_flow()))
    }
}

impl Routable for TimeCommand {
    fn route(self) -> Route {
        match self.action {
            Some(action) => action.route(),
            None => Route::Session(Box::new(self)),
        }
    }
}

impl SessionCommand for TimeCommand {
    fn answer(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
