use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use crate::command::{LocalCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigGetCommand {
    #[usage(complete = complete_config_key)]
    pub key: Option<String>,
    #[usage(
        long,
        help = "Show the keys below the key with their types and defaults instead of their values"
    )]
    pub schema: bool,
}

impl ConfigGetCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &ConfigStore<T>) -> anyhow::Result<String> {
        let key = self.key.as_deref().unwrap_or_default();

        if self.schema { store.schema(key) } else { store.get(key) }
    }
}

impl Routable for ConfigGetCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigGet(self))
    }
}
