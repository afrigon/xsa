use xsa_commands::command::InterfaceSetCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceSetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.set_screen(self.screen);

        Ok(format!("ui: set {:?}", self.screen))
    }
}
