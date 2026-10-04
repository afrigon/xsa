use super::{TransitionAppearance, TransitionRole};

// An animated change between two screens. `progress` runs from 0 to 1 over `duration` seconds, linearly; a
// transition eases it as it likes.
pub trait Transition: 'static {
    fn duration(&self) -> f32;

    fn appearance(&self, progress: f32, role: TransitionRole) -> TransitionAppearance;

    // Called every frame the transition runs, ending with 1, so a transition can drive things outside the
    // interface, like a camera.
    fn progressed(&mut self, _progress: f32) {}
}
