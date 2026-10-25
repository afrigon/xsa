use usage::Args;
use xsa_proto::request::{Request, SetTimeRateRequest};
use xsa_proto::session::SessionState;
use xsa_units::TimeRate;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimePauseCommand {}

impl Routable for TimePauseCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimePauseCommand {
    fn request(&self) -> Request {
        Request::SetTimeRate(SetTimeRateRequest { rate: TimeRate::PAUSED })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
