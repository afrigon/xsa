use crate::Point;

// How a screen appears at one moment of a transition.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TransitionAppearance {
    pub opacity: f32,
    pub offset: Point,
}

impl TransitionAppearance {
    pub const IDENTITY: TransitionAppearance = TransitionAppearance {
        opacity: 1.0,
        offset: Point { x: 0.0, y: 0.0 },
    };
}
