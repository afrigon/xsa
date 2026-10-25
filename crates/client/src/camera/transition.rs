mod view_point;
mod zoom_path;
mod zoom_path_shape;

use std::f64::consts::SQRT_2;

use animation::{EasedProgress, Easing};
use glam::{DQuat, DVec3};

use super::{Anchor, AnyControls, CameraPose, Controls, ControlsContext};
use crate::app::ClientWorld;
use crate::input::Input;
use view_point::ViewPoint;
use zoom_path::ZoomPath;

const SECONDS_PER_PATH_LENGTH: f64 = 0.25;
const MINIMUM_SECONDS: f64 = 0.75;
const MAXIMUM_SECONDS: f64 = 2.5;
const ARC_CURVATURE: f64 = SQRT_2;

// Carries the camera from the controls it left to the ones it took along a zoom-and-pan path. Both keep updating,
// the ones it left without input, so either end follows the body it is about while the camera travels.
pub struct CameraTransition {
    from: AnyControls,
    path: ZoomPath,
    start_from: ViewPoint,
    start_to: ViewPoint,
    progress: EasedProgress,
}

impl CameraTransition {
    pub fn new(
        from: AnyControls,
        from_pose: CameraPose,
        to_anchor: Anchor,
        to_pose: CameraPose,
        world: &ClientWorld,
        duration_scale: f64,
    ) -> CameraTransition {
        let start_from = ViewPoint::between(from.anchor().position(world), from_pose.position);
        let start_to = ViewPoint::between(to_anchor.position(world), to_pose.position);
        let path = ZoomPath::new(start_from, start_to, ARC_CURVATURE);
        let duration = (path.length * SECONDS_PER_PATH_LENGTH).clamp(MINIMUM_SECONDS, MAXIMUM_SECONDS) * duration_scale;

        CameraTransition {
            from,
            path,
            start_from,
            start_to,
            progress: EasedProgress::new(duration, Easing::Smoothstep),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.progress.is_finished()
    }

    // Advances the transition and blends the left controls' pose into `to_pose`.
    pub fn update(&mut self, to_pose: CameraPose, to_anchor: Anchor, context: &ControlsContext) -> CameraPose {
        self.progress.advance(context.delta_seconds);
        let idle = Input::default();
        let from_pose = self.from.update(&ControlsContext {
            input: &idle,
            ..*context
        });
        let progress = self.progress.value();
        let remaining = 1.0 - progress;
        let from = ViewPoint::between(self.from.anchor().position(context.world), from_pose.position);
        let to = ViewPoint::between(to_anchor.position(context.world), to_pose.position);
        let on_path = self.path.at(progress);
        let focus = on_path.focus
            + (from.focus - self.start_from.focus) * remaining
            + (to.focus - self.start_to.focus) * progress;
        let distance = on_path.distance
            * (from.distance / self.start_from.distance).powf(remaining)
            * (to.distance / self.start_to.distance).powf(progress);
        let from_direction = (from_pose.position - from.focus).normalize_or(DVec3::Y);
        let to_direction = (to_pose.position - to.focus).normalize_or(DVec3::Y);
        let turn = DQuat::IDENTITY.slerp(DQuat::from_rotation_arc(from_direction, to_direction), progress);

        CameraPose {
            position: focus + turn * from_direction * distance,
            orientation: from_pose.orientation.slerp(to_pose.orientation, progress),
        }
    }
}
