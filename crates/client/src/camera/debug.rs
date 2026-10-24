use std::f64::consts::FRAC_PI_2;

use glam::{DQuat, DVec3};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use xsa_core::simulation::BodyIndex;

use super::{Anchor, Attitude, CameraPose, Controls, ControlsContext};
use crate::app::ClientWorld;
use crate::config::DebugCameraConfig;
use crate::input::Input;

const MOUSE_SENSITIVITY: f64 = 0.0015;
const SPEED_FACTOR_PER_SCROLL_STEP: f64 = 2.0;

// Flies freely with WASD, Space and Shift and mouse look, riding the nearest body's motion and spin.
pub struct DebugControls {
    position: DVec3,
    // Turns the camera's local axes into the local horizontal frame, whose +Z is up. Each update tilts it by the
    // smallest rotation onto the new up, so the heading carries over smoothly as the camera travels.
    frame: DQuat,
    yaw: f64,
    // Stops at straight up and down, as in first-person games: past them the view would turn upside down.
    pitch: f64,
    // A view to take once the frame matches the next update's up.
    pending_view: Option<PendingView>,
    // The body the camera rides on, as it stood on the previous frame.
    attachment: Attachment,
}

enum PendingView {
    Direction(DVec3),
    Orientation(DQuat),
}

#[derive(Clone, Copy)]
struct Attachment {
    body: BodyIndex,
    position: DVec3,
    orientation: DQuat,
}

impl DebugControls {
    // Stands at `position` riding `body`, facing `orientation`'s yaw and pitch.
    pub fn new(position: DVec3, orientation: DQuat, body: BodyIndex, world: &ClientWorld) -> DebugControls {
        DebugControls {
            position,
            frame: DQuat::IDENTITY,
            yaw: 0.0,
            pitch: 0.0,
            pending_view: Some(PendingView::Orientation(orientation)),
            attachment: Attachment::of(body, world),
        }
    }

    // Places the camera at `position`, riding `body` from there.
    pub fn stand(&mut self, position: DVec3, body: BodyIndex, world: &ClientWorld) {
        self.position = position;
        self.attachment = Attachment::of(body, world);
    }

    pub fn look_along(&mut self, direction: DVec3) {
        self.pending_view = Some(PendingView::Direction(direction));
    }

    // The speed scrolling asks for, if it scrolled.
    pub fn scrolled_speed(input: &Input, debug_camera: &DebugCameraConfig) -> Option<f64> {
        let steps = input.scroll_steps();

        (steps != 0.0).then(|| f64::from(debug_camera.speed) * SPEED_FACTOR_PER_SCROLL_STEP.powf(steps))
    }

    // Carries the camera along with its body's motion and spin since the previous frame, so it hovers over the same
    // ground whatever the simulation's speed, then rides whichever body is nearest from there. Carrying comes first:
    // after a jump in time, the position the body left behind says nothing about which body is near.
    fn ride_nearest_body(&mut self, world: &ClientWorld) {
        let state = world.body_state(self.attachment.body);
        let rotation = state.orientation * self.attachment.orientation.inverse();
        self.position = state.position + rotation * (self.position - self.attachment.position);
        self.frame = (rotation * self.frame).normalize();
        let nearest = world.nearest_body(self.position).unwrap_or(self.attachment.body);
        self.attachment = Attachment::of(nearest, world);
    }

    fn take_pending_view(&mut self) {
        match self.pending_view.take() {
            Some(PendingView::Direction(direction)) => {
                let local = self.frame.inverse() * direction.normalize();
                self.yaw = (-local.x).atan2(local.y);
                self.pitch = local.z.clamp(-1.0, 1.0).asin();
            }
            Some(PendingView::Orientation(orientation)) => {
                let attitude = Attitude::of(self.frame.inverse() * orientation, DVec3::Z, DVec3::Y);
                self.yaw = attitude.yaw;
                self.pitch = attitude.pitch;
            }
            None => {}
        }
    }
}

impl Controls for DebugControls {
    fn update(&mut self, context: &ControlsContext) -> CameraPose {
        let input = context.input;
        self.ride_nearest_body(context.world);
        let up = (self.position - self.attachment.position).normalize_or(DVec3::Z);
        self.frame = (DQuat::from_rotation_arc(self.frame * DVec3::Z, up) * self.frame).normalize();
        self.take_pending_view();

        let mouse_delta = input.mouse_delta();
        self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-FRAC_PI_2, FRAC_PI_2);

        let heading = self.frame * DQuat::from_rotation_z(self.yaw);
        let forward = heading * DVec3::Y;
        let right = heading * DVec3::X;
        let direction = forward * input.axis(KeyCode::KeyW, KeyCode::KeyS)
            + right * input.axis(KeyCode::KeyD, KeyCode::KeyA)
            + up * input.axis(KeyCode::Space, KeyCode::ShiftLeft);
        self.position += direction.normalize_or_zero() * f64::from(context.debug_camera.speed) * context.delta_seconds;

        CameraPose {
            position: self.position,
            orientation: heading * DQuat::from_rotation_x(self.pitch),
        }
    }

    fn anchor(&self) -> Anchor {
        Anchor::Body(self.attachment.body)
    }

    fn wants_mouse_capture(&self, input: &Input) -> Option<bool> {
        input.was_button_pressed(MouseButton::Left).then_some(true)
    }

    fn name(&self) -> &'static str {
        "debug"
    }

    fn speed(&self, debug_camera: &DebugCameraConfig) -> Option<f64> {
        Some(f64::from(debug_camera.speed))
    }
}

impl Attachment {
    fn of(body: BodyIndex, world: &ClientWorld) -> Attachment {
        let state = world.body_state(body);

        Attachment {
            body,
            position: state.position,
            orientation: state.orientation,
        }
    }
}
