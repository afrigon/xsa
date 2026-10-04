use std::any::Any;

use super::{Presentation, ViewController};
use crate::AnyView;

// A view controller behind one type, so a stack can hold controllers of different types. Every view controller
// implements it.
pub trait AnyViewController: Any {
    fn root_view(&self) -> AnyView;

    fn presentation(&self) -> Presentation;

    fn did_load(&mut self);

    fn did_unload(&mut self);

    fn did_appear(&mut self);

    fn did_disappear(&mut self);
}

impl<Controller: ViewController> AnyViewController for Controller {
    fn root_view(&self) -> AnyView {
        AnyView::new(ViewController::root(self))
    }

    fn presentation(&self) -> Presentation {
        ViewController::presentation(self)
    }

    fn did_load(&mut self) {
        ViewController::did_load(self);
    }

    fn did_unload(&mut self) {
        ViewController::did_unload(self);
    }

    fn did_appear(&mut self) {
        ViewController::did_appear(self);
    }

    fn did_disappear(&mut self) {
        ViewController::did_disappear(self);
    }
}

impl dyn AnyViewController {
    // Whether this is a `Controller`, for delegates choosing a transition per pair of controllers.
    pub fn is<Controller: ViewController>(&self) -> bool {
        (self as &dyn Any).is::<Controller>()
    }
}

impl<Controller: ViewController> From<Controller> for Box<dyn AnyViewController> {
    fn from(controller: Controller) -> Box<dyn AnyViewController> {
        Box::new(controller)
    }
}
