use glam::DVec2;
use xsa_commands::command::InputCursorSetCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputCursorSetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.inject_cursor_position(DVec2::new(self.x, self.y))?;

        Ok(format!("input: cursor at {}, {}", self.x, self.y))
    }
}
