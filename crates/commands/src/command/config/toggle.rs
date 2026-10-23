use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use super::ConfigSetCommand;
use crate::command::{LocalCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigToggleCommand {
    #[usage(complete = complete_config_key)]
    pub key: String,
    #[usage(long, help = "Write the change to the config file in effect")]
    pub save: bool,
}

impl ConfigToggleCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        store.toggle(&self.key, self.save)?;

        ConfigSetCommand::describe(store, &self.key, self.save)
    }
}

impl Routable for ConfigToggleCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigToggle(self))
    }
}
