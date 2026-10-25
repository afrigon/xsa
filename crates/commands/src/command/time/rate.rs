use usage::Args;
use xsa_proto::request::{Request, SetTimeRateRequest};
use xsa_proto::session::SessionState;
use xsa_units::TimeRate;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimeRateCommand {
    #[usage(allow_negative_numbers)]
    pub multiplier: f64,
}

impl Routable for TimeRateCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeRateCommand {
    fn request(&self) -> Request {
        Request::SetTimeRate(SetTimeRateRequest {
            rate: TimeRate {
                multiplier: self.multiplier,
            },
        })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
