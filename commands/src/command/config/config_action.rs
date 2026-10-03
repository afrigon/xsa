use usage::Subcommands;

use super::{ConfigGetCommand, ConfigReloadCommand, ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum ConfigAction {
    /// Show a setting, every setting below a prefix such as render.exposure, or all of them
    Get(ConfigGetCommand),
    /// Change a setting, e.g. config set render.bloom.strength 0.05
    Set(ConfigSetCommand),
    /// Flip an on/off setting or step to the next choice
    Toggle(ConfigToggleCommand),
    /// Write the settings to the config file
    Save(ConfigSaveCommand),
    /// Read the config file again
    Reload(ConfigReloadCommand),
}

impl Routable for ConfigAction {
    fn route(self) -> Route {
        match self {
            ConfigAction::Get(command) => command.route(),
            ConfigAction::Set(command) => command.route(),
            ConfigAction::Toggle(command) => command.route(),
            ConfigAction::Save(command) => command.route(),
            ConfigAction::Reload(command) => command.route(),
        }
    }
}
