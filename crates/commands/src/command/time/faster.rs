use usage::Args;
use xsa_proto::request::{Request, StepTimeRateRequest};
use xsa_proto::session::SessionState;
use xsa_units::TimeRateStep;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimeFasterCommand {}

impl Routable for TimeFasterCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeFasterCommand {
    fn request(&self) -> Request {
        Request::StepTimeRate(StepTimeRateRequest {
            step: TimeRateStep::Faster,
        })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
