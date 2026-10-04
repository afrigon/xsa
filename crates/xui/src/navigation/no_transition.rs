use super::{Transition, TransitionAppearance, TransitionRole};

// Switches screens at once.
pub struct NoTransition;

impl Transition for NoTransition {
    fn duration(&self) -> f32 {
        0.0
    }

    fn appearance(&self, _progress: f32, _role: TransitionRole) -> TransitionAppearance {
        TransitionAppearance::IDENTITY
    }
}
