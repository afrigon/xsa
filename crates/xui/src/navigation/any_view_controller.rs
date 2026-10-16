use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use super::{Controller, ControllerMessage, Presentation, ViewController};
use crate::{AnyView, Environment};

// A view controller behind one type, so a stack can hold controllers of different types. Every view controller
// implements it.
pub trait AnyViewController: Any {
    fn root_view(&self, id: u64, messages: &Rc<RefCell<Vec<ControllerMessage>>>) -> AnyView;

    fn presentation(&self) -> Presentation;

    fn did_load(&mut self, environment: &Environment);

    fn did_unload(&mut self, environment: &Environment);

    fn did_appear(&mut self, environment: &Environment);

    fn did_disappear(&mut self, environment: &Environment);
}

impl<Implementation: ViewController> AnyViewController for Implementation {
    fn root_view(&self, id: u64, messages: &Rc<RefCell<Vec<ControllerMessage>>>) -> AnyView {
        AnyView::new(ViewController::root(self, &Controller::new(id, messages.clone())))
    }

    fn presentation(&self) -> Presentation {
        ViewController::presentation(self)
    }

    fn did_load(&mut self, environment: &Environment) {
        ViewController::did_load(self, environment);
    }

    fn did_unload(&mut self, environment: &Environment) {
        ViewController::did_unload(self, environment);
    }

    fn did_appear(&mut self, environment: &Environment) {
        ViewController::did_appear(self, environment);
    }

    fn did_disappear(&mut self, environment: &Environment) {
        ViewController::did_disappear(self, environment);
    }
}

impl dyn AnyViewController {
    // Whether this is a `Controller`, for delegates choosing a transition per pair of controllers.
    pub fn is<Implementation: ViewController>(&self) -> bool {
        (self as &dyn Any).is::<Implementation>()
    }
}

impl<Implementation: ViewController> From<Implementation> for Box<dyn AnyViewController> {
    fn from(controller: Implementation) -> Box<dyn AnyViewController> {
        Box::new(controller)
    }
}
