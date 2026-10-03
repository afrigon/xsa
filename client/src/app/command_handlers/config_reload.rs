use xsa_commands::command::ConfigReloadCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for ConfigReloadCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.reload()?;
        app.reconfigure();

        Ok(format!("reloaded {}", app.config_document.path().display()))
    }
}
