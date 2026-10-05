use xui::{AnyViewController, DefaultNavigationDelegate, NavigationDelegate, NavigationOperation, Transition};

use super::{GameViewController, MainMenuViewController, MenuPresence, ZoomTransition};

pub struct GameNavigationDelegate {
    pub presence: MenuPresence,
}

impl NavigationDelegate for GameNavigationDelegate {
    fn transition(
        &self,
        operation: NavigationOperation,
        from: &dyn AnyViewController,
        to: &dyn AnyViewController,
    ) -> Box<dyn Transition> {
        if from.is::<MainMenuViewController>() && to.is::<GameViewController>() {
            return Box::new(ZoomTransition {
                presence: self.presence.clone(),
            });
        }

        DefaultNavigationDelegate.transition(operation, from, to)
    }
}
