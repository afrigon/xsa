use std::f64::consts::{FRAC_PI_2, PI, TAU};

use anyhow::{Context, bail, ensure};
use glam::{DMat3, DQuat, DVec3};
use winit::dpi::PhysicalSize;
use xsa_commands::command::{CameraTargetCommand, CameraTeleportCommand, TargetSelection};
use xsa_commands::value::{Latitude, Longitude};
use xsa_core::simulation::BodyIndex;

use super::body_step::BodyStep;
use super::client_world::ClientWorld;
use crate::camera::{
    Anchor, AnyControls, Attitude, Camera, CameraPose, CameraSwitch, CameraTransition, Controls, ControlsContext,
    DebugControls, OrbitControls,
};
use crate::config::{CameraConfig, DebugCameraConfig};
use crate::input::Input;
use crate::ui::CameraReading;

const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const METERS_PER_KILOMETER: f64 = 1_000.0;
const FULL_TURN_DEGREES: f64 = 360.0;
// Replies give angles to a thousandth of a degree.
const ANGLE_PRECISION: f64 = 1000.0;

pub(super) struct CameraController {
    camera: Camera,
    // None until the simulation has a body to look at.
    controls: Option<AnyControls>,
    transition: Option<CameraTransition>,
    // Where the controls put the camera, before a transition blends it in: where the camera is about to be.
    controls_pose: CameraPose,
    // The last switch of controls, until the app takes it to release the mouse and, after a cut, the frame history.
    switched: Option<CameraSwitch>,
    debug_camera: DebugCameraConfig,
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

impl CameraController {
    pub fn new(window_size: PhysicalSize<u32>, config: &CameraConfig) -> CameraController {
        CameraController {
            camera: Camera::new(
                HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians(),
                CameraController::aspect_ratio_of(window_size),
                config.collision,
            ),
            controls: None,
            transition: None,
            controls_pose: CameraPose {
                position: DVec3::ZERO,
                orientation: DQuat::IDENTITY,
            },
            switched: None,
            debug_camera: config.debug.clone(),
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

    pub fn controls_pose(&self) -> CameraPose {
        self.controls_pose
    }

    pub fn anchor(&self) -> Option<Anchor> {
        self.controls.as_ref().map(AnyControls::anchor)
    }

    pub fn reading(&self) -> Option<CameraReading> {
        let controls = self.controls.as_ref()?;

        Some(CameraReading {
            controls: controls.name(),
            speed: controls.speed(&self.debug_camera),
            attitude: Attitude::of(self.camera.orientation, DVec3::Z, DVec3::Y),
        })
    }

    pub fn focus(&mut self, world: &ClientWorld) {
        if let Some(body) = world.initial_target() {
            let controls = OrbitControls::new(body, world.body(body).radius, DVec3::Y);
            self.switch(AnyControls::Orbit(controls), CameraSwitch::Cut, world);
        }
    }

    pub fn set_animation_duration_scale(&mut self, animation_duration_scale: f64) {
        self.animation_duration_scale = animation_duration_scale;
    }

    pub fn take_switch(&mut self) -> Option<CameraSwitch> {
        self.switched.take()
    }

    pub fn set_camera_config(&mut self, config: &CameraConfig) {
        self.camera.collision = config.collision;
        self.debug_camera = config.debug.clone();
    }

    pub fn wants_mouse_capture(&self, input: &Input) -> Option<bool> {
        self.controls.as_ref()?.wants_mouse_capture(input)
    }

    // Returns the debug camera's speed that scrolling asked for, if it did.
    pub fn update(&mut self, world: &ClientWorld, input: &Input, delta_seconds: f64) -> Option<f64> {
        let controls = self.controls.as_mut()?;
        let context = ControlsContext {
            world,
            input,
            camera: &self.camera,
            debug_camera: &self.debug_camera,
            delta_seconds,
        };
        let mut pose = controls.update(&context);
        self.controls_pose = pose;

        if let Some(transition) = &mut self.transition {
            pose = transition.update(pose, controls.anchor(), &context);

            if transition.is_finished() {
                self.transition = None;
            }
        }

        let scrolled_speed = match controls {
            AnyControls::Debug(_) => DebugControls::scrolled_speed(input, &self.debug_camera),
            AnyControls::Orbit(_) => None,
        };
        self.camera.position = pose.position;
        self.camera.orientation = pose.orientation;

        scrolled_speed
    }

    pub fn target(&mut self, command: &CameraTargetCommand, world: &ClientWorld) -> anyhow::Result<String> {
        let current = self.anchor().context("the simulation has no bodies")?.body();
        let target = match command.selection()? {
            TargetSelection::Body { id } => world.find_body(&id)?,
            TargetSelection::Next { category } => world.step_body(current, BodyStep::Next, category)?,
            TargetSelection::Previous { category } => world.step_body(current, BodyStep::Previous, category)?,
            TargetSelection::Parent => world
                .body(current)
                .parent
                .with_context(|| format!("{} orbits nothing", world.body(current).id))?,
        };
        let body = world.body(target);

        if let Some(AnyControls::Orbit(orbit)) = &mut self.controls
            && orbit.body() == target
        {
            return CameraController::aim(orbit, command, world, &self.camera);
        }

        let forward = self.camera.orientation * DVec3::Y;
        let mut orbit = OrbitControls::new(target, body.radius, forward);
        let reply = CameraController::aim(&mut orbit, command, world, &self.camera)?;
        self.switch(AnyControls::Orbit(orbit), CameraSwitch::Smooth, world);

        Ok(reply)
    }

    // Applies the command's pitch, yaw and altitude to the orbit, replying with where it looks from.
    fn aim(
        orbit: &mut OrbitControls,
        command: &CameraTargetCommand,
        world: &ClientWorld,
        camera: &Camera,
    ) -> anyhow::Result<String> {
        let target = orbit.body();
        let body = world.body(target);

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
            let closest = OrbitControls::closest_distance(body.radius, camera);
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
    // Every value left out keeps the camera's current one over the body the camera is about by default.
    pub fn teleport(
        &mut self,
        command: &CameraTeleportCommand,
        world: &ClientWorld,
    ) -> anyhow::Result<(TeleportTarget, String)> {
        let index = match &command.body {
            Some(body) => world.find_body(body)?,
            None => self.anchor().context("the camera is around no body; name one")?.body(),
        };
        let current = world.position_reading(index, self.camera.position, self.camera.orientation);
        let current_surface = || {
            current.surface.as_ref().with_context(|| {
                format!(
                    "the camera is at the centre of {}, with no latitude, longitude or heading to keep; give --lat, \
                     --lon, --yaw and --pitch",
                    world.body(index).id
                )
            })
        };
        let latitude = match command.latitude {
            Some(latitude) => CameraController::latitude(latitude, index, command, world)?,
            None => current_surface()?.latitude,
        };
        let longitude = match command.longitude {
            Some(longitude) => CameraController::longitude(longitude, latitude, index, command, world)?,
            None => current_surface()?.longitude,
        };
        let altitude = command.altitude.map_or_else(
            || {
                current
                    .surface
                    .as_ref()
                    .and_then(|surface| surface.above_ground)
                    .unwrap_or(current.above_sea)
            },
            |altitude| altitude.meters,
        );
        // The debug camera never rolls, so the current view is kept by its direction: a compass heading and the angle
        // above the horizon.
        let forward = self.camera.orientation * DVec3::Y;
        let current_horizon = || -> anyhow::Result<DMat3> {
            let surface = current_surface()?;
            let up = (self.camera.position - world.body_state(index).position).normalize();
            let north = world.local_north(index, surface.latitude, surface.longitude);

            Ok(DMat3::from_cols(north.cross(up), north, up))
        };
        let yaw = match command.yaw {
            Some(yaw) => yaw.to_radians(),
            None => {
                let horizon = current_horizon()?;
                forward.dot(horizon.x_axis).atan2(forward.dot(horizon.y_axis))
            }
        };
        let pitch = match command.pitch {
            Some(pitch) => pitch.to_radians(),
            None => forward.dot(current_horizon()?.z_axis).clamp(-1.0, 1.0).asin(),
        };

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
        let up = state.orientation * direction;
        let north = world.local_north(index, latitude, longitude);
        let east = north.cross(up);
        let horizon = DQuat::from_mat3(&DMat3::from_cols(east, north, up));
        let orientation = horizon * DQuat::from_rotation_z(-yaw) * DQuat::from_rotation_x(pitch);
        let position = world.eye_position(index, direction, altitude);
        let controls = DebugControls::new(position, orientation, index, world);
        self.switch(AnyControls::Debug(controls), CameraSwitch::Smooth, world);

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

    fn latitude(
        latitude: Latitude,
        body: BodyIndex,
        command: &CameraTeleportCommand,
        world: &ClientWorld,
    ) -> anyhow::Result<f64> {
        let declination = |other| world.sky_position(body, other).declination;

        match latitude {
            Latitude::Degrees { degrees } => {
                ensure!(
                    degrees.abs() <= FRAC_PI_2.to_degrees(),
                    "a latitude lies between -90 and 90 degrees, not {degrees}"
                );

                Ok(degrees.to_radians())
            }
            Latitude::Substellar => Ok(declination(world.star_for(body, command.star.as_deref())?)),
            Latitude::Antistellar => Ok(-declination(world.star_for(body, command.star.as_deref())?)),
            Latitude::NearSide => Ok(declination(CameraController::parent(body, world)?)),
            Latitude::FarSide => Ok(-declination(CameraController::parent(body, world)?)),
        }
    }

    fn longitude(
        longitude: Longitude,
        latitude: f64,
        body: BodyIndex,
        command: &CameraTeleportCommand,
        world: &ClientWorld,
    ) -> anyhow::Result<f64> {
        let overhead = |other| world.sky_position(body, other).overhead_longitude;

        match longitude {
            Longitude::Degrees { degrees } => Ok(degrees.to_radians()),
            Longitude::TimeOfDay { time_of_day } => {
                let star = world.star_for(body, command.star.as_deref())?;

                Ok(world
                    .daylight(body, star)
                    .longitude(time_of_day, latitude, world.time())?)
            }
            Longitude::NearSide => Ok(overhead(CameraController::parent(body, world)?)),
            Longitude::FarSide => {
                let opposite = overhead(CameraController::parent(body, world)?) + PI;

                Ok((opposite + PI).rem_euclid(TAU) - PI)
            }
        }
    }

    fn parent(body: BodyIndex, world: &ClientWorld) -> anyhow::Result<BodyIndex> {
        world
            .body(body)
            .parent
            .with_context(|| format!("{} orbits nothing, so it has no near or far side", world.body(body).id))
    }

    // Places the debug camera at `target` against the ground loaded there so far.
    pub fn stand(&mut self, target: TeleportTarget, world: &ClientWorld) -> anyhow::Result<()> {
        let Some(AnyControls::Debug(debug)) = &mut self.controls else {
            bail!("the camera left the debug controls while standing");
        };
        let position = world.eye_position(target.body, target.direction, target.altitude);
        debug.stand(position, target.body, world);

        Ok(())
    }

    pub fn look_at(&mut self, id: &str, world: &ClientWorld) -> anyhow::Result<String> {
        let Some(AnyControls::Debug(debug)) = &mut self.controls else {
            bail!("look-at turns the debug camera; stand somewhere with `camera tp` first");
        };
        let body = world.find_body(id)?;
        let direction = (world.body_state(body).position - self.camera.position).normalize_or_zero();
        ensure!(direction != DVec3::ZERO, "the camera is at the center of {id}");

        debug.look_along(direction);

        Ok(format!("looking at {id}"))
    }

    // Replaces the controls; a smooth switch carries the camera from where it is now.
    fn switch(&mut self, mut controls: AnyControls, switch: CameraSwitch, world: &ClientWorld) {
        let idle = Input::default();
        let to_pose = controls.update(&ControlsContext {
            world,
            input: &idle,
            camera: &self.camera,
            debug_camera: &self.debug_camera,
            delta_seconds: 0.0,
        });
        let to_anchor = controls.anchor();
        let previous = self.controls.replace(controls);
        let switch = if self.animation_duration_scale > 0.0 {
            switch
        } else {
            CameraSwitch::Cut
        };
        self.controls_pose = to_pose;
        self.transition = None;
        self.switched = Some(switch);

        let (Some(previous), CameraSwitch::Smooth) = (previous, switch) else {
            return;
        };
        let from_pose = CameraPose {
            position: self.camera.position,
            orientation: self.camera.orientation,
        };
        self.transition = Some(CameraTransition::new(
            previous,
            from_pose,
            to_anchor,
            to_pose,
            world,
            self.animation_duration_scale,
        ));
    }

    fn aspect_ratio_of(window_size: PhysicalSize<u32>) -> f32 {
        window_size.width as f32 / window_size.height as f32
    }
}
