use super::PointerEventKind;
use crate::Point;

// What the pointer did, at `position`: in physical pixels when given to the interface, in logical points when
// offered to views.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PointerEvent {
    pub kind: PointerEventKind,
    pub position: Point,
}
