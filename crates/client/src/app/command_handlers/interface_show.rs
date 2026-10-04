use xsa_commands::command::InterfaceShowCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceShowCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.set_overlay_shown(self.overlay, true);

        Ok(format!("ui: showing {:?}", self.overlay))
    }
}
