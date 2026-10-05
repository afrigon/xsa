use usage::Args;
use xsa_proto::message::{ClientMessage, SetTimeRate};
use xsa_proto::session::SessionState;
use xsa_units::TimeRate;

use super::TimeCommand;
use crate::command::{Availability, Routable, Route, ServerCommand};
use crate::router::Output;

#[derive(Args)]
pub struct TimePauseCommand {}

impl Routable for TimePauseCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }

    fn availability(&self) -> Availability {
        Availability::InGame
    }
}

impl ServerCommand for TimePauseCommand {
    fn message(&self) -> ClientMessage {
        ClientMessage::SetTimeRate(SetTimeRate { rate: TimeRate::PAUSED })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
