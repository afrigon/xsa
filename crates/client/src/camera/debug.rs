use std::f64::consts::FRAC_PI_2;

use glam::{DQuat, DVec3};
use winit::keyboard::KeyCode;

use super::{Attitude, Camera};
use crate::input::Input;

const MOUSE_SENSITIVITY: f64 = 0.0015;
const SPEED_FACTOR_PER_SCROLL_STEP: f64 = 2.0;

pub struct DebugCamera {
    // Turns the camera's local axes into the local horizontal frame, whose +Z is up. Each update tilts it by the
    // smallest rotation onto the new up, so the heading carries over smoothly as the camera travels.
    frame: DQuat,
    yaw: f64,
    // Stops at straight up and down, as in first-person games: past them the view would turn upside down.
    pitch: f64,
    speed: f64,
    // A view to take once the frame matches the next update's up.
    pending_view: Option<PendingView>,
}

enum PendingView {
    Direction(DVec3),
    Orientation(DQuat),
}

impl DebugCamera {
    pub fn new(speed: f64) -> Self {
        Self {
            frame: DQuat::IDENTITY,
            yaw: 0.0,
            pitch: 0.0,
            speed,
            pending_view: None,
        }
    }

    // Turns the horizontal frame with the body the camera rides on.
    pub fn carry(&mut self, rotation: DQuat) {
        self.frame = (rotation * self.frame).normalize();
    }

    // Meters per second.
    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn look_along(&mut self, direction: DVec3) {
        self.pending_view = Some(PendingView::Direction(direction));
    }

    // Takes a world orientation's yaw and pitch; the camera never rolls.
    pub fn face(&mut self, orientation: DQuat) {
        self.pending_view = Some(PendingView::Orientation(orientation));
    }

    // `up` is the local vertical, a unit vector.
    pub fn update(&mut self, camera: &mut Camera, input: &Input, up: DVec3, delta_seconds: f64) {
        self.frame = (DQuat::from_rotation_arc(self.frame * DVec3::Z, up) * self.frame).normalize();

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

        let mouse_delta = input.mouse_delta();
        self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-FRAC_PI_2, FRAC_PI_2);
        self.speed *= SPEED_FACTOR_PER_SCROLL_STEP.powf(input.scroll_steps());

        let heading = self.frame * DQuat::from_rotation_z(self.yaw);
        let forward = heading * DVec3::Y;
        let right = heading * DVec3::X;
        let direction = forward * input.axis(KeyCode::KeyW, KeyCode::KeyS)
            + right * input.axis(KeyCode::KeyD, KeyCode::KeyA)
            + up * input.axis(KeyCode::Space, KeyCode::ShiftLeft);

        camera.position += direction.normalize_or_zero() * self.speed * delta_seconds;
        camera.orientation = heading * DQuat::from_rotation_x(self.pitch);
    }
}
