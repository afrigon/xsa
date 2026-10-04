mod binding;
mod state_slots;

pub use binding::Binding;
pub(crate) use state_slots::StateSlots;

use std::cell::RefCell;
use std::rc::Rc;

// A value a view owns across frames, like SwiftUI's `@State`. Cloning it shares the same value.
pub struct State<T: 'static> {
    value: Rc<RefCell<T>>,
}

impl<T: 'static> State<T> {
    pub(crate) fn new(value: Rc<RefCell<T>>) -> State<T> {
        State { value }
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
    }

    pub fn update(&self, change: impl FnOnce(&mut T)) {
        change(&mut self.value.borrow_mut());
    }
}

impl<T: Clone + 'static> State<T> {
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    // A handle a child view can read and write, like SwiftUI's `$value`.
    pub fn binding(&self) -> Binding<T> {
        let read = self.value.clone();
        let write = self.value.clone();

        Binding::new(move || read.borrow().clone(), move |value| *write.borrow_mut() = value)
    }
}

impl<T: 'static> Clone for State<T> {
    fn clone(&self) -> State<T> {
        State {
            value: self.value.clone(),
        }
    }
}
