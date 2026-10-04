use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use crate::State;

// The states of one view, each keyed by where `Context::state` was called for it, so a condition around one call
// does not shift the others.
#[derive(Default)]
pub(crate) struct StateSlots {
    values: RefCell<HashMap<&'static Location<'static>, Rc<dyn Any>>>,
}

impl StateSlots {
    pub fn state<T: 'static>(&self, location: &'static Location<'static>, initial: impl FnOnce() -> T) -> State<T> {
        let mut values = self.values.borrow_mut();
        let value = values
            .entry(location)
            .or_insert_with(|| Rc::new(RefCell::new(initial())))
            .clone();

        match value.downcast::<RefCell<T>>() {
            Ok(value) => State::new(value),
            Err(_) => unreachable!("a call site always asks for the same type"),
        }
    }
}
