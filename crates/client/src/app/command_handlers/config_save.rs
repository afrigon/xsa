use xsa_commands::command::ConfigSaveCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for ConfigSaveCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.save()?;

        Ok(format!("saved to {}", app.config_document.path().display()))
    }
}
