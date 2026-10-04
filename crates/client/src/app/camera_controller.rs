use anyhow::{Context, bail, ensure};
use glam::DVec3;
use winit::event::MouseButton;
use xsa_commands::command::CameraTargetCommand;
use xsa_commands::value::Target;
use xsa_core::simulation::BodyIndex;

use super::body_step::BodyStep;
use super::client_world::ClientWorld;
use crate::camera::debug::DebugCamera;
use crate::camera::orbit::{OrbitCamera, OrbitTarget};
use crate::camera::{Camera, CameraMode};
use crate::input::Input;

const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const DEBUG_CAMERA_SPEED: f64 = 2_000_000.0;
const METERS_PER_KILOMETER: f64 = 1_000.0;

pub(super) struct CameraController {
    camera: Camera,
    mode: CameraMode,
    orbit: Option<OrbitCamera>,
    debug: DebugCamera,
}

impl CameraController {
    pub fn new() -> CameraController {
        CameraController {
            camera: Camera::new(HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians()),
            mode: CameraMode::Orbit,
            orbit: None,
            debug: DebugCamera::new(DEBUG_CAMERA_SPEED),
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn target_body(&self) -> Option<BodyIndex> {
        self.orbit.as_ref().map(OrbitCamera::target)
    }

    pub fn focus(&mut self, world: &ClientWorld) {
        self.orbit = world
            .initial_target()
            .map(|target| OrbitCamera::new(target, world.body(target).radius));
    }

    // Returns whether the mode changed.
    pub fn set_mode(&mut self, mode: CameraMode) -> bool {
        if self.mode == mode {
            return false;
        }

        if mode == CameraMode::Debug
            && let Some(orbit) = &self.orbit
        {
            self.debug.look_along(orbit.yaw(), orbit.pitch());
        }

        self.mode = mode;

        true
    }

    pub fn wants_mouse_capture(&self, input: &Input) -> Option<bool> {
        match self.mode {
            CameraMode::Debug if input.was_button_pressed(MouseButton::Left) => Some(true),
            CameraMode::Orbit if input.was_button_pressed(MouseButton::Right) => Some(true),
            CameraMode::Orbit if input.was_button_released(MouseButton::Right) => Some(false),
            _ => None,
        }
    }

    pub fn update(&mut self, world: &ClientWorld, input: &Input, delta_seconds: f64) {
        match self.mode {
            CameraMode::Orbit => {
                if let Some(orbit) = &mut self.orbit {
                    let target = orbit.target();
                    let target = OrbitTarget {
                        position: world.body_state(target).position,
                        radius: world.body(target).radius,
                    };
                    orbit.update(&mut self.camera, input, target, delta_seconds);
                }
            }
            CameraMode::Debug => self.debug.update(&mut self.camera, input, delta_seconds),
        }

        let nearest_surface_distance = world.nearest_surface_distance(self.camera.position);
        self.camera.fit_near_plane(nearest_surface_distance);
    }

    pub fn target(&mut self, command: &CameraTargetCommand, world: &ClientWorld) -> anyhow::Result<String> {
        let orbit = self.orbit.as_mut().context("the simulation has no bodies")?;
        let current = orbit.target();
        let steps = matches!(command.target, Target::Next | Target::Previous);
        ensure!(
            steps || command.category.is_none(),
            "a category only applies to next and previous"
        );

        let target = match &command.target {
            Target::Next => world.step_body(current, BodyStep::Next, command.category)?,
            Target::Previous => world.step_body(current, BodyStep::Previous, command.category)?,
            Target::Parent => world
                .body(current)
                .parent
                .with_context(|| format!("{} orbits nothing", world.body(current).id))?,
            Target::Body { id } => world.find_body(id)?,
        };
        let body = world.body(target);

        if target != orbit.target() {
            orbit.set_target(target, world.body_state(target).position, body.radius);
        }

        if let Some(distance) = command.distance {
            orbit.set_distance(distance.meters);
        }

        if let Some(pitch) = command.pitch {
            orbit.set_pitch(pitch.to_radians());
        }

        if let Some(yaw) = command.yaw {
            orbit.set_yaw(yaw.to_radians());
        }

        Ok(format!(
            "target: {}, distance {:.0} km, pitch {:.1}°, yaw {:.1}°",
            body.id,
            orbit.distance() / METERS_PER_KILOMETER,
            orbit.pitch().to_degrees(),
            orbit.yaw().to_degrees()
        ))
    }

    pub fn look_at(&mut self, target: &Target, world: &ClientWorld) -> anyhow::Result<String> {
        ensure!(
            self.mode == CameraMode::Debug,
            "look-at turns the debug camera; switch to it with `camera mode debug`"
        );

        let Target::Body { id } = target else {
            bail!("look-at needs a body id");
        };
        let body = world.find_body(id)?;
        let direction = (world.body_state(body).position - self.camera.position).normalize_or_zero();
        ensure!(direction != DVec3::ZERO, "the camera is at the center of {id}");

        self.debug
            .look_along((-direction.x).atan2(direction.y), direction.z.asin());

        Ok(format!("looking at {id}"))
    }
}
