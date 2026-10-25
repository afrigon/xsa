use memory_vulkan::Timeline;

/// A queue of a family that can encode video, and the timeline its submissions signal.
pub struct EncodeQueue {
    pub family: u32,
    pub timeline: Timeline,
}
