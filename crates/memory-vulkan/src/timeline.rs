mod timeline_id;
mod timeline_semaphore;

pub(crate) use timeline_id::TimelineId;
pub(crate) use timeline_semaphore::TimelineSemaphore;

use ash::vk;

use crate::{MemoryContext, MemoryError};

/// A queue's timeline semaphore: every submission to the queue goes through it and signals the next value, so the
/// deletion queue knows what the GPU may still be using. It lives as long as its `MemoryContext`.
#[derive(Clone)]
pub struct Timeline {
    memory: MemoryContext,
    id: TimelineId,
}

impl Timeline {
    pub(crate) fn new(memory: MemoryContext, id: TimelineId) -> Timeline {
        Timeline { memory, id }
    }

    /// Submits to the timeline's queue and returns the value the submission signals once it completes.
    pub fn submit(
        &self,
        waits: &[vk::SemaphoreSubmitInfo],
        command_buffers: &[vk::CommandBufferSubmitInfo],
        signals: &[vk::SemaphoreSubmitInfo],
    ) -> Result<u64, MemoryError> {
        self.memory.submit(self.id, waits, command_buffers, signals)
    }

    /// Blocks until the GPU has completed the submission that signals `value`.
    pub fn wait(&self, value: u64) -> Result<(), MemoryError> {
        self.memory.wait_timeline(self.id, value)
    }
}
