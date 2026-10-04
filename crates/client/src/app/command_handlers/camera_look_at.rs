use anyhow::Context;
use xsa_commands::command::CameraLookAtCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for CameraLookAtCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let world = app.world.as_ref().context("not joined to a simulation yet")?;

        app.cameras.look_at(&self.target, world)
    }
}
