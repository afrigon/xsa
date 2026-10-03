use xsa_commands::command::ConfigToggleCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;

impl ClientCommandHandler for ConfigToggleCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.config_document.toggle(&self.key)?;

        app.apply_config_change(&self.key, self.save)
    }
}
