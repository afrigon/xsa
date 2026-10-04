use super::Transition;

// A transition in progress: the view controllers it moves between, the ones visible before it started, and the entries
// it removes when it ends.
pub(crate) struct ActiveTransition {
    pub transition: Box<dyn Transition>,
    pub elapsed: f32,
    pub outgoing: u64,
    pub incoming: u64,
    pub visible_before: Vec<u64>,
    pub removed: Vec<u64>,
}

impl ActiveTransition {
    pub fn progress(&self) -> f32 {
        let duration = self.transition.duration();

        if duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / duration).min(1.0)
        }
    }
}
