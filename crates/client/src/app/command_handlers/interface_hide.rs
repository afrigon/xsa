use xsa_commands::command::InterfaceHideCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceHideCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.set_overlay_shown(self.overlay, false);

        Ok(format!("ui: hiding {:?}", self.overlay))
    }
}
