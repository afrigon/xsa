use crate::Timeline;

/// The extra queue a `TestDevice` created for its `TestRequirements`.
pub struct TestQueue {
    pub family: u32,
    pub timeline: Timeline,
}
