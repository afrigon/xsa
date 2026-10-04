use xsa_commands::command::ConfigToggleCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for ConfigToggleCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.toggle(&self.key)?;

        app.apply_config_change(&self.key, self.save)
    }
}
