use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct ConfigSaveCommand {}

impl Routable for ConfigSaveCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigSave(self))
    }
}
