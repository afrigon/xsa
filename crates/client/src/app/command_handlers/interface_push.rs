use xsa_commands::command::InterfacePushCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfacePushCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.push_view_controller(self.view_controller);

        Ok(format!("ui: pushed {:?}", self.view_controller))
    }
}
