use glam::DVec2;
use xsa_commands::command::InputCursorMoveCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputCursorMoveCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.inject_mouse_offset(DVec2::new(self.dx, self.dy))?;

        Ok(format!("input: moved the mouse by {}, {}", self.dx, self.dy))
    }
}
