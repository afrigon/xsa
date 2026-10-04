use super::PointerButton;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerEventKind {
    Moved,
    Pressed { button: PointerButton },
    Released { button: PointerButton },
    // The pointer left the window.
    Left,
}
