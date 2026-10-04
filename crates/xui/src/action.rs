use std::rc::Rc;

// Something a view can trigger, like a Swift closure handed to a SwiftUI view: `perform` for actions taking
// nothing, `perform_with` for actions taking a value, such as the id of a selected item. Cloning it shares it.
pub struct Action<Input: 'static = ()> {
    perform: Rc<dyn Fn(Input)>,
}

impl<Input: 'static> Action<Input> {
    pub fn new(perform: impl Fn(Input) + 'static) -> Action<Input> {
        Action {
            perform: Rc::new(perform),
        }
    }

    pub fn perform_with(&self, input: Input) {
        (self.perform)(input);
    }
}

impl Action {
    pub fn perform(&self) {
        self.perform_with(());
    }

    // An action that does nothing.
    pub fn none() -> Action {
        Action::new(|()| {})
    }
}

impl<Input: 'static> Clone for Action<Input> {
    fn clone(&self) -> Action<Input> {
        Action {
            perform: self.perform.clone(),
        }
    }
}
