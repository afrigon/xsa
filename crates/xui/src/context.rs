use std::panic::Location;
use std::rc::Rc;

use crate::state::StateSlots;
use crate::{Environment, State};

// What a view's body can read and keep, like the property wrappers of a SwiftUI view: the environment, and state
// stored in the view's node.
pub struct Context {
    environment: Environment,
    states: Rc<StateSlots>,
}

impl Context {
    pub(crate) fn new(environment: Environment, states: Rc<StateSlots>) -> Context {
        Context { environment, states }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    // The view's state for this call site, created with `initial` the first time, like `@State var x = initial`.
    #[track_caller]
    pub fn state<T: 'static>(&self, initial: impl FnOnce() -> T) -> State<T> {
        self.states.state(Location::caller(), initial)
    }
}
