use xsa_config::{ConfigSection, ConfigStore};

#[cfg(feature = "client")]
use super::InputBindCommand;
use super::{
    ConfigGetCommand, ConfigLoadCommand, ConfigSaveCommand, ConfigSetCommand, ConfigToggleCommand, ConfigUnsetCommand,
};

pub enum LocalCommand {
    ConfigGet(ConfigGetCommand),
    ConfigSet(ConfigSetCommand),
    ConfigToggle(ConfigToggleCommand),
    ConfigUnset(ConfigUnsetCommand),
    ConfigSave(ConfigSaveCommand),
    ConfigLoad(ConfigLoadCommand),
    #[cfg(feature = "client")]
    InputBind(InputBindCommand),
}

impl LocalCommand {
    pub fn run<T: ConfigSection + Default + Clone>(self, store: &mut ConfigStore<T>) -> anyhow::Result<String> {
        match self {
            LocalCommand::ConfigGet(command) => command.run(store),
            LocalCommand::ConfigSet(command) => command.run(store),
            LocalCommand::ConfigToggle(command) => command.run(store),
            LocalCommand::ConfigUnset(command) => command.run(store),
            LocalCommand::ConfigSave(command) => command.run(store),
            LocalCommand::ConfigLoad(command) => command.run(store),
            #[cfg(feature = "client")]
            LocalCommand::InputBind(command) => command.run(store),
        }
    }
}
