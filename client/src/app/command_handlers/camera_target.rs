use anyhow::Context;
use xsa_commands::command::CameraTargetCommand;
use xsa_commands::value::Target;

use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::{App, METERS_PER_KILOMETER};
use crate::camera::CameraMode;

impl ClientCommandHandler for CameraTargetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let orbit_camera = app.orbit_camera.as_mut().context("the simulation has no bodies")?;
        let bodies = world.simulation.bodies();
        let target = match &self.target {
            Target::Next => (orbit_camera.target() + 1) % bodies.len(),
            Target::Previous => (orbit_camera.target() + bodies.len() - 1) % bodies.len(),
            Target::Body { id } => world.find_body(id)?,
        };
        let body = &bodies[target];

        if target != orbit_camera.target() {
            orbit_camera.set_target(target, world.state.bodies[target].position, body.radius);
        }

        if let Some(distance) = self.distance {
            orbit_camera.set_distance(distance.meters);
        }

        if let Some(pitch) = self.pitch {
            orbit_camera.set_pitch(pitch.to_radians());
        }

        if let Some(yaw) = self.yaw {
            orbit_camera.set_yaw(yaw.to_radians());
        }

        let description = format!(
            "target: {}, distance {:.0} km, pitch {:.1}°, yaw {:.1}°",
            body.id,
            orbit_camera.distance() / METERS_PER_KILOMETER,
            orbit_camera.pitch().to_degrees(),
            orbit_camera.yaw().to_degrees()
        );
        app.set_camera_mode(CameraMode::Orbit);

        Ok(description)
    }
}
