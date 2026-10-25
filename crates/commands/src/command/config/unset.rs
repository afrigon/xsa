use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use crate::command::{LocalCommand, Routable, Route};
use crate::completion::complete_config_key;

#[derive(Args)]
pub struct ConfigUnsetCommand {
    #[usage(complete = complete_config_key)]
    pub key: String,
    #[usage(long, help = "Also remove the setting from the config file in effect")]
    pub save: bool,
}

impl ConfigUnsetCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        store.unset(&self.key, self.save)?;
        let mut description = match store.get(&self.key) {
            Ok(value) => format!("{} {value}", self.key),
            Err(_) => format!("{} is back to its defaults", self.key),
        };

        if self.save {
            description.push_str(&format!(" (removed from {})", store.path().display()));
        }

        Ok(description)
    }
}

impl Routable for ConfigUnsetCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigUnset(self))
    }
}
