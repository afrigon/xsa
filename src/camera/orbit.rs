use std::f64::consts::FRAC_PI_2;

use glam::{DQuat, DVec3};
use winit::event::MouseButton;

use super::Camera;
use crate::input::Input;

const MOUSE_SENSITIVITY: f64 = 0.004;
const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
const ZOOM_FACTOR_PER_SCROLL_STEP: f64 = 1.25;
const MINIMUM_ALTITUDE: f64 = 100.0;
const MAXIMUM_DISTANCE: f64 = 1e13;
const FRAMING_DISTANCE_IN_RADII: f64 = 4.0;
const TRANSITION_SECONDS: f64 = 1.25;
const ARC_DISTANCE_PER_TRAVELED_DISTANCE: f64 = 0.75;

pub struct OrbitTarget {
    pub position: DVec3,
    pub radius: f64,
}

struct Transition {
    start_focus: DVec3,
    start_distance: f64,
    elapsed: f64,
}

pub struct OrbitCamera {
    pub smooth_transitions: bool,
    pub arc_transitions: bool,
    target: usize,
    distance: f64,
    yaw: f64,
    pitch: f64,
    transition: Option<Transition>,
    current_focus: DVec3,
    current_distance: f64,
}

impl OrbitCamera {
    pub fn new(target: usize, target_radius: f64) -> Self {
        let distance = target_radius * FRAMING_DISTANCE_IN_RADII;
        Self {
            smooth_transitions: true,
            arc_transitions: true,
            target,
            distance,
            yaw: 0.0,
            pitch: 0.0,
            transition: None,
            current_focus: DVec3::ZERO,
            current_distance: distance,
        }
    }

    pub fn target(&self) -> usize {
        self.target
    }

    pub fn yaw(&self) -> f64 {
        self.yaw
    }

    pub fn pitch(&self) -> f64 {
        self.pitch
    }

    pub fn set_target(&mut self, target: usize, target_radius: f64) {
        self.transition = self.smooth_transitions.then_some(Transition {
            start_focus: self.current_focus,
            start_distance: self.current_distance,
            elapsed: 0.0,
        });
        self.target = target;
        self.distance = target_radius * FRAMING_DISTANCE_IN_RADII;
    }

    pub fn update(&mut self, camera: &mut Camera, input: &Input, target: OrbitTarget, delta_seconds: f64) {
        if input.is_button_held(MouseButton::Right) {
            let mouse_delta = input.mouse_delta();
            self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
            self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
        self.distance = (self.distance / ZOOM_FACTOR_PER_SCROLL_STEP.powf(input.scroll_steps()))
            .clamp(target.radius + MINIMUM_ALTITUDE, MAXIMUM_DISTANCE);

        let (focus, distance) = match &mut self.transition {
            Some(transition) => {
                transition.elapsed += delta_seconds;
                let progress = ease_in_out((transition.elapsed / TRANSITION_SECONDS).min(1.0));
                let focus = transition.start_focus.lerp(target.position, progress);
                let start = transition.start_distance.ln();
                let end = self.distance.ln();
                let mut log_distance = start + (end - start) * progress;
                if self.arc_transitions {
                    let traveled = transition.start_focus.distance(target.position);
                    let context = (traveled * ARC_DISTANCE_PER_TRAVELED_DISTANCE).max(f64::MIN_POSITIVE).ln();
                    let rise = (context - (start + end) / 2.0).max(0.0);
                    log_distance += rise * 4.0 * progress * (1.0 - progress);
                }
                (focus, log_distance.exp())
            }
            None => (target.position, self.distance),
        };
        if self
            .transition
            .as_ref()
            .is_some_and(|transition| transition.elapsed >= TRANSITION_SECONDS)
        {
            self.transition = None;
        }
        self.current_focus = focus;
        self.current_distance = distance;

        let orientation = DQuat::from_rotation_z(self.yaw) * DQuat::from_rotation_x(self.pitch);
        camera.orientation = orientation;
        camera.position = focus - orientation * DVec3::Y * distance;
    }
}

fn ease_in_out(progress: f64) -> f64 {
    progress * progress * (3.0 - 2.0 * progress)
}
