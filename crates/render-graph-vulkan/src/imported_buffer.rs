use ash::vk;

use crate::BufferState;

/// A buffer owned outside the graph, such as a readback buffer, and the states it starts and must end the frame in.
#[derive(Clone, Copy, Debug)]
pub struct ImportedBuffer {
    pub name: &'static str,
    pub buffer: vk::Buffer,
    pub initial: BufferState,
    pub final_state: BufferState,
}
