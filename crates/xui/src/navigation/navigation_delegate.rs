use super::{AnyViewController, NavigationOperation, Transition};

// Chooses the transition between two screens, like UIKit's `animationControllerFor:from:to:`.
pub trait NavigationDelegate: 'static {
    fn transition(
        &self,
        operation: NavigationOperation,
        from: &dyn AnyViewController,
        to: &dyn AnyViewController,
    ) -> Box<dyn Transition>;
}
