use xsa_commands::command::ConfigGetCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;

impl ClientCommandHandler for ConfigGetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.get(self.key.as_deref().unwrap_or_default())
    }
}
