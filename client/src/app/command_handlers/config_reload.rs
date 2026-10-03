use xsa_commands::command::ConfigReloadCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;

impl ClientCommandHandler for ConfigReloadCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.reload()?;
        app.reconfigure();

        Ok(format!("reloaded {}", app.config_document.path().display()))
    }
}
