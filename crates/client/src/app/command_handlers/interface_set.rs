use xsa_commands::command::InterfaceSetCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceSetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.set_view_controller(self.view_controller);

        Ok(format!("ui: set {:?}", self.view_controller))
    }
}
