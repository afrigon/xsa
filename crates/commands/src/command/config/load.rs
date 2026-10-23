use std::path::PathBuf;

use anyhow::bail;
use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use super::Settings;
use crate::command::{LocalCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigLoadCommand {
    pub path: Option<PathBuf>,
    #[usage(long, complete = complete_config_key, help = "Only the settings below this key, read relative to it")]
    pub key: Option<String>,
    #[usage(long, help = "Also write the loaded settings to the config file in effect")]
    pub save: bool,
}

impl ConfigLoadCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        let key = self.key.as_deref().unwrap_or_default();

        let Some(path) = self.path else {
            if self.save {
                bail!("--save needs a file to load from");
            }

            store.reload(key)?;

            return Ok(format!("reloaded {}", store.path().display()));
        };

        let count = store.import(&path, key, self.save)?;
        let mut description = format!("loaded {} from {}", Settings(count), path.display());

        if self.save {
            description.push_str(&format!(" (saved to {})", store.path().display()));
        }

        Ok(description)
    }
}

impl Routable for ConfigLoadCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigLoad(self))
    }
}
