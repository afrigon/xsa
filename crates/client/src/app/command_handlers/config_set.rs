use xsa_commands::command::ConfigSetCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for ConfigSetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.set(&self.key, &self.value)?;

        app.apply_config_change(&self.key, self.save)
    }
}
