use usage::Args;
use xsa_config::{ConfigSection, ConfigStore};

use crate::command::{LocalCommand, Routable, Route};
use crate::completion::{complete_config_key, complete_config_value};

#[derive(Args)]
pub struct ConfigSetCommand {
    #[usage(complete = complete_config_key)]
    pub key: String,
    #[usage(allow_negative_numbers, complete = complete_config_value)]
    pub value: String,
    #[usage(long, help = "Write the change to the config file in effect")]
    pub save: bool,
}

impl ConfigSetCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        store.set(&self.key, &self.value, self.save)?;

        ConfigSetCommand::describe(store, &self.key, self.save)
    }

    pub(super) fn describe<T: ConfigSection + Default + Clone>(
        store: &ConfigStore<T>,
        key: &str,
        save: bool,
    ) -> anyhow::Result<String> {
        let mut description = format!("{key} {}", store.get(key)?);

        if save {
            description.push_str(&format!(" (saved to {})", store.path().display()));
        }

        Ok(description)
    }
}

impl Routable for ConfigSetCommand {
    fn route(self) -> Route {
        Route::Local(LocalCommand::ConfigSet(self))
    }
}
