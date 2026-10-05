use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};

#[derive(Args)]
pub struct ConfigSaveCommand {}

impl Routable for ConfigSaveCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigSave(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
