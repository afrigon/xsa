use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};
use crate::completion::{complete_config_key, complete_config_value};

#[derive(Args)]
pub struct ConfigSetCommand {
    #[usage(complete = complete_config_key)]
    pub key: String,
    #[usage(allow_negative_numbers, complete = complete_config_value)]
    pub value: String,
    #[usage(long, help = "Also write the change to the config file")]
    pub save: bool,
}

impl Routable for ConfigSetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::ConfigSet(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
