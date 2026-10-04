use super::{AnyViewController, FadeTransition, NavigationDelegate, NavigationOperation, Presentation, Transition};

const FADE_SECONDS: f32 = 0.25;

// A quick fade of the view controllers that appear or disappear: pushing an overlay fades only it in, popping one fades
// only it out.
pub struct DefaultNavigationDelegate;

impl NavigationDelegate for DefaultNavigationDelegate {
    fn transition(
        &self,
        operation: NavigationOperation,
        from: &dyn AnyViewController,
        to: &dyn AnyViewController,
    ) -> Box<dyn Transition> {
        let (fades_outgoing, fades_incoming) = match operation {
            NavigationOperation::Push => (to.presentation() == Presentation::FullScreen, true),
            NavigationOperation::Pop => (true, from.presentation() == Presentation::FullScreen),
            NavigationOperation::SetRoot => (true, true),
        };

        Box::new(FadeTransition {
            duration: FADE_SECONDS,
            fades_outgoing,
            fades_incoming,
        })
    }
}
