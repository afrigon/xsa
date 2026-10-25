use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InputResetCommand {}

impl Routable for InputResetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputReset(self))
    }
}
