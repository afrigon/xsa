use usage::Args;
use xsa_proto::request::{Request, StepTimeRequest};
use xsa_proto::session::SessionState;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;
use crate::value::Duration;

#[derive(Args)]
pub struct TimeStepCommand {
    pub duration: Duration,
}

impl Routable for TimeStepCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeStepCommand {
    fn request(&self) -> Request {
        Request::StepTime(StepTimeRequest {
            duration: self.duration.duration(),
        })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
