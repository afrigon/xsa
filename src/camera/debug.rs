use std::f64::consts::FRAC_PI_2;

use glam::{DQuat, DVec3};
use winit::keyboard::KeyCode;

use super::Camera;
use crate::input::Input;

const MOUSE_SENSITIVITY: f64 = 0.002;
const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
const SPEED_FACTOR_PER_SCROLL_STEP: f64 = 2.0;

pub struct DebugCamera {
    yaw: f64,
    pitch: f64,
    speed: f64,
}

impl DebugCamera {
    pub fn new(speed: f64) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, input: &Input, delta_seconds: f64) {
        let mouse_delta = input.mouse_delta();
        self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        self.speed *= SPEED_FACTOR_PER_SCROLL_STEP.powf(input.scroll_steps());

        let heading = DQuat::from_rotation_z(self.yaw);
        let forward = heading * DVec3::Y;
        let right = heading * DVec3::X;
        let direction = forward * key_axis(input, KeyCode::KeyW, KeyCode::KeyS)
            + right * key_axis(input, KeyCode::KeyD, KeyCode::KeyA)
            + DVec3::Z * key_axis(input, KeyCode::Space, KeyCode::ShiftLeft);

        camera.position += direction.normalize_or_zero() * self.speed * delta_seconds;
        camera.orientation = heading * DQuat::from_rotation_x(self.pitch);
    }
}

fn key_axis(input: &Input, positive: KeyCode, negative: KeyCode) -> f64 {
    let value = |key| if input.is_held(key) { 1.0 } else { 0.0 };
    value(positive) - value(negative)
}
