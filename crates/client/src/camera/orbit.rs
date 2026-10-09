mod orbit_target;
mod transition;
mod view_point;
mod zoom_path;
mod zoom_path_shape;

pub use orbit_target::OrbitTarget;

use std::f64::consts::{FRAC_PI_2, SQRT_2};

use glam::{DQuat, DVec3};
use winit::event::MouseButton;
use xsa_core::simulation::BodyIndex;

use super::Camera;
use crate::input::Input;
use transition::Transition;
use view_point::ViewPoint;
use zoom_path::ZoomPath;

const MOUSE_SENSITIVITY: f64 = 0.004;
const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
const ZOOM_FACTOR_PER_SCROLL_STEP: f64 = 1.25;
const FRAMING_MARGIN: f64 = 1.05;
const MAXIMUM_DISTANCE: f64 = 1e13;
const FRAMING_DISTANCE_IN_RADII: f64 = 4.0;
const TRANSITION_SECONDS_PER_PATH_LENGTH: f64 = 0.25;
const MINIMUM_TRANSITION_SECONDS: f64 = 0.75;
const MAXIMUM_TRANSITION_SECONDS: f64 = 2.5;
const ARC_CURVATURE: f64 = SQRT_2;

pub struct OrbitCamera {
    target: BodyIndex,
    distance: f64,
    yaw: f64,
    pitch: f64,
    transition: Option<Transition>,
    current: ViewPoint,
}

impl OrbitCamera {
    pub fn new(target: BodyIndex, target_radius: f64) -> Self {
        let distance = target_radius * FRAMING_DISTANCE_IN_RADII;
        Self {
            target,
            distance,
            yaw: 0.0,
            pitch: 0.0,
            transition: None,
            current: ViewPoint {
                focus: DVec3::ZERO,
                distance,
            },
        }
    }

    pub fn target(&self) -> BodyIndex {
        self.target
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

    pub fn set_distance(&mut self, distance: f64) {
        self.distance = distance.min(MAXIMUM_DISTANCE);
    }

    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;
    }

    pub fn set_target(&mut self, target: BodyIndex, target_position: DVec3, target_radius: f64) {
        self.target = target;
        self.distance = target_radius * FRAMING_DISTANCE_IN_RADII;
        let end = ViewPoint {
            focus: target_position,
            distance: self.distance,
        };
        let path = ZoomPath::new(self.current, end, ARC_CURVATURE);
        let duration = (path.length * TRANSITION_SECONDS_PER_PATH_LENGTH)
            .clamp(MINIMUM_TRANSITION_SECONDS, MAXIMUM_TRANSITION_SECONDS);
        self.transition = Some(Transition {
            path,
            end,
            elapsed: 0.0,
            duration,
        });
    }

    pub fn update(&mut self, camera: &mut Camera, input: &Input, target: OrbitTarget, delta_seconds: f64) {
        if input.is_button_held(MouseButton::Right) {
            let mouse_delta = input.mouse_delta();
            self.yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
            self.pitch = (self.pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }

        let minimum_distance = target.radius * FRAMING_MARGIN / f64::from(camera.narrowest_half_fov()).sin();
        self.distance = (self.distance / ZOOM_FACTOR_PER_SCROLL_STEP.powf(input.scroll_steps()))
            .clamp(minimum_distance, MAXIMUM_DISTANCE);

        self.current = match &mut self.transition {
            Some(transition) => {
                transition.elapsed += delta_seconds;
                let progress = transition.eased_progress();
                let on_path = transition.path.at(progress);
                let target_drift = target.position - transition.end.focus;
                let zoom_adjustment = (self.distance / transition.end.distance).powf(progress);
                ViewPoint {
                    focus: on_path.focus + target_drift * progress,
                    distance: on_path.distance * zoom_adjustment,
                }
            }
            None => ViewPoint {
                focus: target.position,
                distance: self.distance,
            },
        };

        if self
            .transition
            .as_ref()
            .is_some_and(|transition| transition.elapsed >= transition.duration)
        {
            self.transition = None;
        }

        let orientation = DQuat::from_rotation_z(self.yaw) * DQuat::from_rotation_x(self.pitch);
        camera.orientation = orientation;
        camera.position = self.current.focus - orientation * DVec3::Y * self.current.distance;
    }
}
