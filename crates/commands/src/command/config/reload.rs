use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct ConfigReloadCommand {}

impl Routable for ConfigReloadCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigReload(self))
    }
}
