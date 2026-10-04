use xsa_commands::command::InterfacePushCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfacePushCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.push_screen(self.screen);

        Ok(format!("ui: pushed {:?}", self.screen))
    }
}
