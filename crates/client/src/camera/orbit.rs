use std::f64::consts::FRAC_PI_2;

use glam::{DQuat, DVec3};
use winit::event::MouseButton;
use xsa_core::simulation::BodyIndex;

use super::{Anchor, Camera, CameraPose, Controls, ControlsContext};
use crate::config::InputConfig;
use crate::input::Input;

const MOUSE_SENSITIVITY: f64 = 0.004;
const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
const ZOOM_FACTOR_PER_SCROLL_STEP: f64 = 1.25;
const FRAMING_MARGIN: f64 = 1.05;
const MAXIMUM_DISTANCE: f64 = 1e13;
const FRAMING_DISTANCE_IN_RADII: f64 = 4.0;

// Orbits a body: right-drag turns around it, scroll zooms.
pub struct OrbitControls {
    body: BodyIndex,
    distance: f64,
    yaw: f64,
    pitch: f64,
}

impl OrbitControls {
    // Frames the body, looking along `forward`.
    pub fn new(body: BodyIndex, body_radius: f64, forward: DVec3) -> OrbitControls {
        let forward = forward.normalize_or(DVec3::Y);

        OrbitControls {
            body,
            distance: body_radius * FRAMING_DISTANCE_IN_RADII,
            yaw: (-forward.x).atan2(forward.y),
            pitch: forward.z.clamp(-1.0, 1.0).asin().clamp(-PITCH_LIMIT, PITCH_LIMIT),
        }
    }

    pub fn body(&self) -> BodyIndex {
        self.body
    }

    pub fn yaw(&self) -> f64 {
        self.yaw
    }

    pub fn pitch(&self) -> f64 {
        self.pitch
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }

    // From the camera towards the body.
    pub fn view_direction(&self) -> DVec3 {
        self.orientation() * DVec3::Y
    }

    // The orbit keeps the whole body in view, so it never comes closer than this.
    pub fn closest_distance(body_radius: f64, camera: &Camera) -> f64 {
        body_radius * FRAMING_MARGIN / f64::from(camera.narrowest_half_fov()).sin()
    }

    pub fn set_distance(&mut self, distance: f64) {
        self.distance = distance.min(MAXIMUM_DISTANCE);
    }

    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;
    }

    fn orientation(&self) -> DQuat {
        DQuat::from_rotation_z(self.yaw) * DQuat::from_rotation_x(self.pitch)
    }
}

impl Controls for OrbitControls {
    fn update(&mut self, context: &ControlsContext) -> CameraPose {
        let input = context.input;

        if input.is_button_held(MouseButton::Right) {
            let mouse_delta = input.mouse_delta();
            self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
            self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }

        let body_radius = context.world.body(self.body).radius;
        let minimum_distance = OrbitControls::closest_distance(body_radius, context.camera);
        self.distance = (self.distance / ZOOM_FACTOR_PER_SCROLL_STEP.powf(input.scroll_steps()))
            .clamp(minimum_distance, MAXIMUM_DISTANCE);

        let orientation = self.orientation();
        let focus = context.world.body_state(self.body).position;

        CameraPose {
            position: focus - orientation * DVec3::Y * self.distance,
            orientation,
        }
    }

    fn anchor(&self) -> Anchor {
        Anchor::Body(self.body)
    }

    fn wants_mouse_capture(&self, input: &Input) -> Option<bool> {
        if input.was_button_pressed(MouseButton::Right) {
            return Some(true);
        }

        input.was_button_released(MouseButton::Right).then_some(false)
    }

    fn name(&self) -> &'static str {
        "orbit"
    }

    fn speed(&self, _config: &InputConfig) -> Option<f64> {
        None
    }
}
