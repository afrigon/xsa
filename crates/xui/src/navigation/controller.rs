use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::{ControllerMessage, ViewController};
use crate::Action;

// A view controller as its root view's actions see it, like Swift's `[weak self]`: actions built here call the
// controller's own methods, with `&mut self`, once the stack next advances.
pub struct Controller<C: ViewController> {
    id: u64,
    messages: Rc<RefCell<Vec<ControllerMessage>>>,
    controller: PhantomData<C>,
}

impl<C: ViewController> Controller<C> {
    pub(crate) fn new(id: u64, messages: Rc<RefCell<Vec<ControllerMessage>>>) -> Controller<C> {
        Controller {
            id,
            messages,
            controller: PhantomData,
        }
    }

    pub fn action(&self, handler: fn(&mut C)) -> Action {
        self.action_with(move |controller: &mut C, ()| handler(controller))
    }

    pub fn action_with<Input: 'static>(&self, handler: impl Fn(&mut C, Input) + Clone + 'static) -> Action<Input> {
        let id = self.id;
        let messages = self.messages.clone();

        Action::new(move |input: Input| {
            let handler = handler.clone();
            messages.borrow_mut().push(ControllerMessage {
                id,
                call: Box::new(move |controller: &mut dyn Any| {
                    if let Some(controller) = controller.downcast_mut::<C>() {
                        handler(controller, input);
                    }
                }),
            });
        })
    }
}
