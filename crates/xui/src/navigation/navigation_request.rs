use super::AnyViewController;

// A change a `Navigation` handle asks of its stack, applied when the stack next advances.
pub(crate) enum NavigationRequest {
    Push(Box<dyn AnyViewController>),
    Pop,
    SetRoot(Box<dyn AnyViewController>),
    Dismiss { id: u64 },
}
