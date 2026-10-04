use std::cell::RefCell;
use std::rc::Rc;

mod active_transition;
mod any_view_controller;
mod controller;
mod controller_message;
mod default_navigation_delegate;
mod dismiss_key;
mod easing;
mod fade_transition;
mod navigation_controller;
mod navigation_delegate;
mod navigation_entry;
mod navigation_layer;
mod navigation_operation;
mod navigation_request;
mod no_transition;
mod presentation;
mod slide_transition;
mod transition;
mod transition_appearance;
mod transition_role;
mod view_controller;

pub(crate) use active_transition::ActiveTransition;
pub use any_view_controller::AnyViewController;
pub use controller::Controller;
pub use controller_message::ControllerMessage;
pub use default_navigation_delegate::DefaultNavigationDelegate;
pub use dismiss_key::DismissKey;
pub use easing::ease_in_out;
pub use fade_transition::FadeTransition;
pub use navigation_controller::NavigationController;
pub use navigation_delegate::NavigationDelegate;
pub(crate) use navigation_entry::NavigationEntry;
pub(crate) use navigation_layer::NavigationLayer;
pub use navigation_operation::NavigationOperation;
pub(crate) use navigation_request::NavigationRequest;
pub use no_transition::NoTransition;
pub use presentation::Presentation;
pub use slide_transition::SlideTransition;
pub use transition::Transition;
pub use transition_appearance::TransitionAppearance;
pub use transition_role::TransitionRole;
pub use view_controller::ViewController;

// A view controller's way to its navigation stack, like UIKit's `navigationController`: a cloneable handle whose
// requests the stack applies when it next advances, never while it is being drawn.
#[derive(Clone)]
pub struct Navigation {
    requests: Rc<RefCell<Vec<NavigationRequest>>>,
}

impl Navigation {
    pub(crate) fn new(requests: Rc<RefCell<Vec<NavigationRequest>>>) -> Navigation {
        Navigation { requests }
    }

    pub fn push(&self, controller: impl Into<Box<dyn AnyViewController>>) {
        self.request(NavigationRequest::Push(controller.into()));
    }

    pub fn pop(&self) {
        self.request(NavigationRequest::Pop);
    }

    pub fn set_root(&self, controller: impl Into<Box<dyn AnyViewController>>) {
        self.request(NavigationRequest::SetRoot(controller.into()));
    }

    pub(crate) fn dismiss(&self, id: u64) {
        self.request(NavigationRequest::Dismiss { id });
    }

    fn request(&self, request: NavigationRequest) {
        self.requests.borrow_mut().push(request);
    }
}
