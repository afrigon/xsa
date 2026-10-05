use xui::{Transition, TransitionAppearance, TransitionRole, ease_in_out};

use super::MenuPresence;

const FADE_FRACTION: f32 = 0.3;

// From the main menu into the game: the camera leaves the menu's framing for the game's while the menu fades out
// first and the game's interface fades in last.
pub struct ZoomTransition {
    pub presence: MenuPresence,
}

impl ZoomTransition {
    pub const DURATION_SECONDS: f32 = 2.5;
}

impl Transition for ZoomTransition {
    fn duration(&self) -> f32 {
        ZoomTransition::DURATION_SECONDS
    }

    fn appearance(&self, progress: f32, role: TransitionRole) -> TransitionAppearance {
        let opacity = match role {
            TransitionRole::Outgoing => 1.0 - ease_in_out(progress / FADE_FRACTION),
            TransitionRole::Incoming => ease_in_out((progress - (1.0 - FADE_FRACTION)) / FADE_FRACTION),
        };

        TransitionAppearance {
            opacity,
            ..TransitionAppearance::IDENTITY
        }
    }

    fn progressed(&mut self, progress: f32) {
        self.presence.set_weight(f64::from(1.0 - ease_in_out(progress)));
    }
}
