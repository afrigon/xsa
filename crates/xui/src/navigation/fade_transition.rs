use super::{Transition, TransitionAppearance, TransitionRole, ease_in_out};

// Cross-fades the screens; a side that stays visible after the transition, like the screen under an overlay,
// is left alone.
pub struct FadeTransition {
    pub duration: f32,
    pub fades_outgoing: bool,
    pub fades_incoming: bool,
}

impl Transition for FadeTransition {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn appearance(&self, progress: f32, role: TransitionRole) -> TransitionAppearance {
        let eased = ease_in_out(progress);
        let opacity = match role {
            TransitionRole::Outgoing if self.fades_outgoing => 1.0 - eased,
            TransitionRole::Incoming if self.fades_incoming => eased,
            _ => 1.0,
        };

        TransitionAppearance {
            opacity,
            ..TransitionAppearance::IDENTITY
        }
    }
}
