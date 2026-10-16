use usage::Args;
use xsa_proto::message::{ClientMessage, SetTimeOverride};
use xsa_proto::session::SessionState;

use super::TimeCommand;
use crate::command::{Routable, Route, ServerCommand};
use crate::router::Output;
use crate::value::TimeOverride;

#[derive(Args)]
pub struct TimeOverrideCommand {
    #[usage(allow_negative_numbers)]
    pub rate: TimeOverride,
}

impl Routable for TimeOverrideCommand {
    fn route(self) -> Route {
        Route::Server(Box::new(self))
    }
}

impl ServerCommand for TimeOverrideCommand {
    fn message(&self) -> ClientMessage {
        ClientMessage::SetTimeOverride(SetTimeOverride { rate: self.rate.rate })
    }

    fn describe(&self, state: Option<&SessionState>) -> Output {
        TimeCommand::describe(state)
    }
}
