use std::path::PathBuf;

use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use crate::command::{LocalCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigSaveCommand {
    pub path: PathBuf,
    #[usage(long, complete = complete_config_key, help = "Only the settings below this key, written relative to it")]
    pub key: Option<String>,
}

impl ConfigSaveCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &ConfigStore<T>) -> anyhow::Result<String> {
        let count = store.export(&self.path, self.key.as_deref().unwrap_or_default())?;

        Ok(format!("saved {count} settings to {}", self.path.display()))
    }
}

impl Routable for ConfigSaveCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigSave(self))
    }
}
