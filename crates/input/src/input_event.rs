use crate::{InputEventKind, InputSource};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputEvent {
    pub source: InputSource,
    pub kind: InputEventKind,
}
