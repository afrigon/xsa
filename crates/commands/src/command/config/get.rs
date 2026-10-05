use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigGetCommand {
    #[usage(complete = complete_config_key)]
    pub key: Option<String>,
}

impl Routable for ConfigGetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigGet(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
