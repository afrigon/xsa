use anyhow::bail;
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
        help = "Show every setting below the key as KDL, each marked session, saved or default"
    )]
    pub dump: bool,
}

impl ConfigGetCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &ConfigStore<T>) -> anyhow::Result<String> {
        match (self.key, self.dump) {
            (key, true) => store.dump(key.as_deref().unwrap_or_default()),
            (Some(key), false) => store.get(&key),
            (None, false) => bail!("name a key, or show every setting with config get --dump"),
        }
    }
}

impl Routable for ConfigGetCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigGet(self))
    }
}
