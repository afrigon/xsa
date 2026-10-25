use crate::Timeline;

/// A queue's family and the timeline its submissions signal.
pub struct QueueTimeline {
    pub family: u32,
    pub timeline: Timeline,
}
