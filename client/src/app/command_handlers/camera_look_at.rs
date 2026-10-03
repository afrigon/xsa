use anyhow::{Context, bail, ensure};
use glam::DVec3;
use xsa_commands::command::CameraLookAtCommand;
use xsa_commands::value::Target;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::camera::CameraMode;

impl ClientCommandHandler for CameraLookAtCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        ensure!(
            app.camera_mode == CameraMode::Debug,
            "look-at turns the debug camera; switch to it with `camera mode debug`"
        );

        let Target::Body { id } = &self.target else {
            bail!("look-at needs a body id");
        };
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let body = world.find_body(id)?;
        let direction = (world.state.bodies[body].position - app.camera.position).normalize_or_zero();
        ensure!(direction != DVec3::ZERO, "the camera is at the center of {id}");

        app.debug_camera
            .look_along((-direction.x).atan2(direction.y), direction.z.asin());

        Ok(format!("looking at {id}"))
    }
}
