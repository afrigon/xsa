use super::{Controller, Presentation};
use crate::View;

// Like UIKit's `UIViewController`: an object that lives as long as it is in a navigation stack. Its root is one
// named view holding the layout, given the actions it triggers; `this` builds actions that call the controller's
// own methods. The hooks tell it when it enters or leaves the stack and when it becomes visible or hidden.
pub trait ViewController: Sized + 'static {
    fn root(&self, this: &Controller<Self>) -> impl View;

    fn presentation(&self) -> Presentation {
        Presentation::FullScreen
    }

    fn did_load(&mut self) {}

    fn did_unload(&mut self) {}

    fn did_appear(&mut self) {}

    fn did_disappear(&mut self) {}
}
