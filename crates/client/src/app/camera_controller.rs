use std::f64::consts::FRAC_PI_2;

use anyhow::{Context, bail, ensure};
use glam::{DMat3, DQuat, DVec3};
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use xsa_commands::command::{CameraTargetCommand, CameraTeleportCommand};
use xsa_commands::value::Target;
use xsa_core::simulation::BodyIndex;

use super::body_step::BodyStep;
use super::client_world::ClientWorld;
use crate::camera::debug::DebugCamera;
use crate::camera::orbit::{OrbitCamera, OrbitTarget};
use crate::camera::{Attitude, Camera, CameraMode};
use crate::input::Input;
use crate::ui::CameraReading;

const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const DEBUG_CAMERA_SPEED: f64 = 2_000_000.0;
const METERS_PER_KILOMETER: f64 = 1_000.0;
const FULL_TURN_DEGREES: f64 = 360.0;
// Replies give angles to a thousandth of a degree.
const ANGLE_PRECISION: f64 = 1000.0;

pub(super) struct CameraController {
    camera: Camera,
    mode: CameraMode,
    orbit: Option<OrbitCamera>,
    debug: DebugCamera,
    // The body the debug camera rides on, as it stood on the previous frame.
    attachment: Option<Attachment>,
    animation_duration_scale: f64,
}

// Where `camera tp` stands the camera: feet `altitude` meters above the ground of `body` toward `direction`, a unit
// vector in the body's frame.
#[derive(Clone, Copy)]
pub(super) struct TeleportTarget {
    pub body: BodyIndex,
    pub direction: DVec3,
    pub altitude: f64,
}

#[derive(Clone, Copy)]
struct Attachment {
    body: BodyIndex,
    position: DVec3,
    orientation: DQuat,
}

impl CameraController {
    pub fn new(window_size: PhysicalSize<u32>) -> CameraController {
        CameraController {
            camera: Camera::new(
                HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians(),
                CameraController::aspect_ratio_of(window_size),
            ),
            mode: CameraMode::Orbit,
            orbit: None,
            debug: DebugCamera::new(DEBUG_CAMERA_SPEED),
            attachment: None,
            animation_duration_scale: 1.0,
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        self.camera.aspect_ratio = CameraController::aspect_ratio_of(window_size);
    }

    pub fn target_body(&self) -> Option<BodyIndex> {
        self.orbit.as_ref().map(OrbitCamera::target)
    }

    // The body the camera is around: the orbited one, or the one the debug camera rides on.
    pub fn reference_body(&self, world: &ClientWorld) -> Option<BodyIndex> {
        match self.mode {
            CameraMode::Orbit => self.target_body(),
            CameraMode::Debug => world.nearest_body(self.camera.position),
        }
    }

    pub fn reading(&self) -> CameraReading {
        CameraReading {
            mode: self.mode,
            speed: (self.mode == CameraMode::Debug).then(|| self.debug.speed()),
            attitude: Attitude::of(self.camera.orientation, DVec3::Z, DVec3::Y),
        }
    }

    pub fn focus(&mut self, world: &ClientWorld) {
        self.orbit = world
            .initial_target()
            .map(|target| OrbitCamera::new(target, world.body(target).radius));
        self.set_animation_duration_scale(self.animation_duration_scale);
    }

    pub fn set_animation_duration_scale(&mut self, animation_duration_scale: f64) {
        self.animation_duration_scale = animation_duration_scale;

        if let Some(orbit) = &mut self.orbit {
            orbit.set_duration_scale(animation_duration_scale);
        }
    }

    // Returns whether the mode changed.
    pub fn set_mode(&mut self, mode: CameraMode) -> bool {
        if self.mode == mode {
            return false;
        }

        if mode == CameraMode::Debug {
            self.debug.face(self.camera.orientation);
            self.attachment = None;
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
            CameraMode::Debug => {
                self.ride_nearest_body(world);
                let up = world.nearest_body(self.camera.position).map_or(DVec3::Z, |body| {
                    (self.camera.position - world.body_state(body).position).normalize()
                });
                self.debug.update(&mut self.camera, input, up, delta_seconds);
            }
        }
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

        if let Some(pitch) = command.pitch {
            orbit.set_pitch(pitch.to_radians());
        }

        if let Some(yaw) = command.yaw {
            orbit.set_yaw(yaw.to_radians());
        }

        let mut note = String::new();
        let state = world.body_state(target);
        let direction = (state.orientation.inverse() * -orbit.view_direction()).normalize();

        if let Some(altitude) = command.altitude {
            let distance = world
                .eye_position(target, direction, altitude.meters)
                .distance(state.position);
            let closest = OrbitCamera::closest_distance(body.radius, &self.camera);
            orbit.set_distance(distance.max(closest));

            if distance < closest {
                note = format!(
                    " (the orbit keeps all of {} in view, so it stops there; `camera tp` reaches the ground)",
                    body.id
                );
            }
        }

        let standing = world.eye_position(target, direction, 0.0).distance(state.position);

        Ok(format!(
            "target: {}, altitude {:.1} km, pitch {:.1}°, yaw {:.1}°{note}",
            body.id,
            (orbit.distance() - standing) / METERS_PER_KILOMETER,
            orbit.pitch().to_degrees(),
            orbit.yaw().to_degrees()
        ))
    }

    // Stands the debug camera's feet `--altitude` above the ground; yaw is a compass heading, pitch above the horizon.
    // Every value left out keeps the camera's current one over the body, `around` by default.
    // Stands the camera where `command` says, returning where with the reply.
    pub fn teleport(
        &mut self,
        command: &CameraTeleportCommand,
        around: Option<BodyIndex>,
        world: &ClientWorld,
    ) -> anyhow::Result<(TeleportTarget, String)> {
        let index = match &command.body {
            Some(body) => world.find_body(body)?,
            None => around.context("the camera is around no body; name one")?,
        };
        let current = world.surface(index, self.camera.position, self.camera.orientation);
        let latitude = command.latitude.map_or(current.latitude, f64::to_radians);
        ensure!(
            latitude.abs() <= FRAC_PI_2,
            "a latitude lies between -90 and 90 degrees, not {}",
            latitude.to_degrees()
        );

        let longitude = command.longitude.map_or(current.longitude, f64::to_radians);
        let altitude = command.altitude.map_or_else(
            || current.above_ground.unwrap_or(current.above_sea),
            |altitude| altitude.meters,
        );
        // The overlay's yaw is counterclockwise; `--yaw` is a compass heading.
        let yaw = command.yaw.map_or(-current.attitude.yaw, f64::to_radians);
        let pitch = command.pitch.map_or(current.attitude.pitch, f64::to_radians);

        let state = world.body_state(index);
        let direction = DVec3::new(
            latitude.cos() * longitude.cos(),
            latitude.cos() * longitude.sin(),
            latitude.sin(),
        );
        let target = TeleportTarget {
            body: index,
            direction,
            altitude,
        };
        self.stand(target, world);

        let up = state.orientation * direction;
        let north = world.local_north(index, latitude, longitude);
        let east = north.cross(up);
        let horizon = DQuat::from_mat3(&DMat3::from_cols(east, north, up));
        self.debug
            .face(horizon * DQuat::from_rotation_z(-yaw) * DQuat::from_rotation_x(pitch));

        let reply = format!(
            "standing on {} at {:.3}, {:.3}, feet {altitude:.1} m above the ground, yaw {:.3}, pitch {:.3}",
            world.body(index).id,
            latitude.to_degrees(),
            longitude.to_degrees(),
            ((yaw.to_degrees() * ANGLE_PRECISION).round() / ANGLE_PRECISION).rem_euclid(FULL_TURN_DEGREES) + 0.0,
            pitch.to_degrees()
        );

        Ok((target, reply))
    }

    // Places the camera at `target` against the ground loaded there so far.
    pub fn stand(&mut self, target: TeleportTarget, world: &ClientWorld) {
        let state = world.body_state(target.body);
        self.camera.position = world.eye_position(target.body, target.direction, target.altitude);
        self.attachment = Some(Attachment {
            body: target.body,
            position: state.position,
            orientation: state.orientation,
        });
    }

    // Carries the camera along with the nearest body's motion and spin since the previous frame, so it hovers over
    // the same ground whatever the simulation's speed.
    fn ride_nearest_body(&mut self, world: &ClientWorld) {
        let Some(index) = world.nearest_body(self.camera.position) else {
            self.attachment = None;
            return;
        };
        let state = world.body_state(index);

        if let Some(attachment) = self.attachment.filter(|attachment| attachment.body == index) {
            let rotation = state.orientation * attachment.orientation.inverse();
            self.camera.position = state.position + rotation * (self.camera.position - attachment.position);
            self.debug.carry(rotation);
        }

        self.attachment = Some(Attachment {
            body: index,
            position: state.position,
            orientation: state.orientation,
        });
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

        self.debug.look_along(direction);

        Ok(format!("looking at {id}"))
    }

    fn aspect_ratio_of(window_size: PhysicalSize<u32>) -> f32 {
        window_size.width as f32 / window_size.height as f32
    }
}
