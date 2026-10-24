use usage::Subcommands;

use super::{
    ConfigGetCommand, ConfigLoadCommand, ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand, ConfigUnsetCommand,
};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum ConfigAction {
    /// Show a setting, or every setting below a key, or their types and defaults with --schema
    Get(ConfigGetCommand),
    /// Change a setting for the session, e.g. config set video.bloom.strength 0.05
    Set(ConfigSetCommand),
    /// Flip an on/off setting or step to the next choice
    Toggle(ConfigToggleCommand),
    /// Return a setting, or every setting below a key, to its default
    Unset(ConfigUnsetCommand),
    /// Export the saved settings to a file
    Save(ConfigSaveCommand),
    /// Read the config file again, or import settings from another file
    Load(ConfigLoadCommand),
}

impl Routable for ConfigAction {
    fn route(self) -> Route {
        match self {
            ConfigAction::Get(command) => command.route(),
            ConfigAction::Set(command) => command.route(),
            ConfigAction::Toggle(command) => command.route(),
            ConfigAction::Unset(command) => command.route(),
            ConfigAction::Save(command) => command.route(),
            ConfigAction::Load(command) => command.route(),
        }
    }
}
