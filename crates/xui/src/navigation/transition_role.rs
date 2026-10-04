// Which side of a transition a screen is on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionRole {
    Outgoing,
    Incoming,
}
