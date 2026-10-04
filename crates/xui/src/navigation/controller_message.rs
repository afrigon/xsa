use std::any::Any;

// A call waiting for the view controller with this id, made by one of its actions.
pub struct ControllerMessage {
    pub(crate) id: u64,
    pub(crate) call: Box<dyn FnOnce(&mut dyn Any)>,
}
