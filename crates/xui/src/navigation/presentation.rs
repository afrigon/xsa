// How a controller covers the ones below it in the stack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Presentation {
    // The controllers below are not drawn while it is on top; they keep their state.
    FullScreen,
    // The controllers below keep drawing under it, like a menu over the game.
    Overlay,
}
