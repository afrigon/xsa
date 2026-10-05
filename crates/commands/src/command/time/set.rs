use usage::Args;
use xsa_proto::message::{ClientMessage, SetTime};
use xsa_proto::session::SessionState;

use super::TimeCommand;
use crate::command::{Availability, Routable, Route, ServerCommand};
use crate::router::Output;
use crate::value::Timestamp;

#[derive(Args)]
pub struct TimeSetCommand {
    pub time: Timestamp,
}

impl Routable for TimeSetCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }

    fn availability(&self) -> Availability {
        Availability::InGame
    }
}

impl ServerCommand for TimeSetCommand {
    fn message(&self) -> ClientMessage {
        ClientMessage::SetTime(SetTime { time: self.time.time })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
