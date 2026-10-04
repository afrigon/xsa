use xsa_commands::command::InterfaceToggleCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceToggleCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let shown = !app.is_overlay_shown(self.overlay);
        app.set_overlay_shown(self.overlay, shown);
        let state = if shown { "showing" } else { "hiding" };

        Ok(format!("ui: {state} {:?}", self.overlay))
    }
}
