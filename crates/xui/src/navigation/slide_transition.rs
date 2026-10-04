use super::{Transition, TransitionAppearance, TransitionRole, ease_in_out};
use crate::Point;

// The incoming screen slides in from `offset` while fading in; the outgoing one slides the other way while fading
// out.
pub struct SlideTransition {
    pub duration: f32,
    pub offset: Point,
}

impl Transition for SlideTransition {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn appearance(&self, progress: f32, role: TransitionRole) -> TransitionAppearance {
        let eased = ease_in_out(progress);
        let (distance, opacity) = match role {
            TransitionRole::Outgoing => (-eased, 1.0 - eased),
            TransitionRole::Incoming => (1.0 - eased, eased),
        };

        TransitionAppearance {
            opacity,
            offset: Point {
                x: self.offset.x * distance,
                y: self.offset.y * distance,
            },
        }
    }
}
