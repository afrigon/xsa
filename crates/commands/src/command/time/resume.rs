use usage::Args;
use xsa_proto::request::{Request, SetTimeRateRequest};
use xsa_proto::session::SessionState;
use xsa_units::{TimeRate, TimeRateChange};

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimeResumeCommand {}

impl Routable for TimeResumeCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeResumeCommand {
    fn request(&self) -> Request {
        Request::SetTimeRate(SetTimeRateRequest {
            change: TimeRateChange::To(TimeRate::REAL_TIME),
        })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
