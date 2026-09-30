use std::f64::consts::{FRAC_PI_2, SQRT_2};

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
const TRANSITION_SECONDS_PER_PATH_LENGTH: f64 = 0.25;
const MINIMUM_TRANSITION_SECONDS: f64 = 0.75;
const MAXIMUM_TRANSITION_SECONDS: f64 = 2.5;
const ARC_CURVATURE: f64 = SQRT_2;
const DIRECT_CURVATURE: f64 = 0.5;

pub struct OrbitTarget {
    pub position: DVec3,
    pub radius: f64,
}

struct Transition {
    path: ZoomPath,
    end_focus: DVec3,
    end_distance: f64,
    elapsed: f64,
    duration: f64,
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

    pub fn set_target(&mut self, target: usize, target_position: DVec3, target_radius: f64) {
        self.target = target;
        self.distance = target_radius * FRAMING_DISTANCE_IN_RADII;
        self.transition = self.smooth_transitions.then(|| {
            let curvature = if self.arc_transitions { ARC_CURVATURE } else { DIRECT_CURVATURE };
            let path = ZoomPath::new(
                (self.current_focus, self.current_distance),
                (target_position, self.distance),
                curvature,
            );
            let duration = (path.length * TRANSITION_SECONDS_PER_PATH_LENGTH)
                .clamp(MINIMUM_TRANSITION_SECONDS, MAXIMUM_TRANSITION_SECONDS);
            Transition {
                path,
                end_focus: target_position,
                end_distance: self.distance,
                elapsed: 0.0,
                duration,
            }
        });
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
                let progress = ease_in_out((transition.elapsed / transition.duration).min(1.0));
                let (path_focus, path_distance) = transition.path.at(progress);
                let target_drift = target.position - transition.end_focus;
                let zoom_adjustment = (self.distance / transition.end_distance).powf(progress);
                (path_focus + target_drift * progress, path_distance * zoom_adjustment)
            }
            None => (target.position, self.distance),
        };
        if self
            .transition
            .as_ref()
            .is_some_and(|transition| transition.elapsed >= transition.duration)
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

// Smooth zoom-and-pan path from van Wijk & Nuij, "Smooth and efficient zooming and panning" (2003).
struct ZoomPath {
    start_focus: DVec3,
    offset: DVec3,
    start_distance: f64,
    curvature: f64,
    shape: ZoomPathShape,
    length: f64,
}

enum ZoomPathShape {
    ZoomOnly,
    Arc { start_angle: f64, travel: f64 },
}

impl ZoomPath {
    fn new((start_focus, start_distance): (DVec3, f64), (end_focus, end_distance): (DVec3, f64), curvature: f64) -> Self {
        let offset = end_focus - start_focus;
        let travel = offset.length();
        let curvature_squared = curvature * curvature;
        let (shape, length) = if travel < f64::EPSILON * start_distance {
            (ZoomPathShape::ZoomOnly, (end_distance / start_distance).ln() / curvature)
        } else {
            let squares = end_distance * end_distance - start_distance * start_distance;
            let pan = curvature_squared * curvature_squared * travel * travel;
            let start_slope = (squares + pan) / (2.0 * start_distance * curvature_squared * travel);
            let end_slope = (squares - pan) / (2.0 * end_distance * curvature_squared * travel);
            let start_angle = -start_slope.asinh();
            let end_angle = -end_slope.asinh();
            (ZoomPathShape::Arc { start_angle, travel }, (end_angle - start_angle) / curvature)
        };
        Self {
            start_focus,
            offset,
            start_distance,
            curvature,
            shape,
            length,
        }
    }

    fn at(&self, progress: f64) -> (DVec3, f64) {
        let position = progress * self.length;
        match self.shape {
            ZoomPathShape::ZoomOnly => (
                self.start_focus + self.offset * progress,
                self.start_distance * (self.curvature * position).exp(),
            ),
            ZoomPathShape::Arc { start_angle, travel } => {
                let angle = self.curvature * position + start_angle;
                let pan_fraction = self.start_distance / (self.curvature * self.curvature * travel)
                    * (start_angle.cosh() * angle.tanh() - start_angle.sinh());
                let distance = self.start_distance * start_angle.cosh() / angle.cosh();
                (self.start_focus + self.offset * pan_fraction, distance)
            }
        }
    }
}
