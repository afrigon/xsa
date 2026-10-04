// Which side of a transition a view controller is on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionRole {
    Outgoing,
    Incoming,
}
