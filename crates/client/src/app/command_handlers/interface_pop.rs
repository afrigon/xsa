use anyhow::ensure;
use xsa_commands::command::InterfacePopCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfacePopCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        ensure!(app.pop_screen(), "the interface is already at its root screen");

        Ok("ui: popped".to_string())
    }
}
