use usage::Args;
use xsa_proto::request::{Request, SetTimeRateRequest};
use xsa_proto::session::SessionState;
use xsa_units::TimeRateChange;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimeSlowerCommand {}

impl Routable for TimeSlowerCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeSlowerCommand {
    fn request(&self) -> Request {
        Request::SetTimeRate(SetTimeRateRequest {
            change: TimeRateChange::Slower,
        })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
