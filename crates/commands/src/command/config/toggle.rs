use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigToggleCommand {
    #[usage(complete = complete_config_key)]
    pub key: String,
    #[usage(long, help = "Also write the change to the config file")]
    pub save: bool,
}

impl Routable for ConfigToggleCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigToggle(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
