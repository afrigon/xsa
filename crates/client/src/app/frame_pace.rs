use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FramePace {
    Wall,
    // Each update covers exactly one step, however long it takes to compute and render.
    Fixed { step: Duration },
}
