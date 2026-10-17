mod recording;
mod timeline_id;
mod timeline_semaphore;
mod timeline_value;

pub use recording::Recording;
pub(crate) use timeline_id::TimelineId;
pub(crate) use timeline_semaphore::TimelineSemaphore;
pub use timeline_value::TimelineValue;

use std::sync::{Arc, Mutex, MutexGuard};

use ash::vk;

use crate::{MemoryContext, MemoryError};

/// A queue's timeline semaphore: every submission to the queue goes through a `Recording` and signals the next value,
/// so the deletion queue knows what the GPU may still be using. It lives as long as its `MemoryContext`, and its clones
/// share the queue, which it keeps externally synchronized across threads.
#[derive(Clone)]
pub struct Timeline {
    memory: MemoryContext,
    id: TimelineId,
    queue: Arc<Mutex<vk::Queue>>,
}

impl Timeline {
    pub(crate) fn new(memory: MemoryContext, id: TimelineId, queue: vk::Queue) -> Timeline {
        Timeline {
            memory,
            id,
            queue: Arc::new(Mutex::new(queue)),
        }
    }

    /// Begins recording `command_buffer`, which must be in the initial state, for a submission to this timeline.
    pub fn begin(&self, command_buffer: vk::CommandBuffer) -> Result<Recording, MemoryError> {
        Recording::begin(self, command_buffer)
    }

    pub(crate) fn submit(
        &self,
        waits: &[vk::SemaphoreSubmitInfo],
        command_buffers: &[vk::CommandBufferSubmitInfo],
        signals: &[vk::SemaphoreSubmitInfo],
    ) -> Result<u64, MemoryError> {
        let queue = self.queue();
        self.memory.submit(self.id, *queue, waits, command_buffers, signals)
    }

    /// Presents through the timeline's queue with a present fence per swapchain, so the swapchain images and semaphores
    /// it uses are known to be released before the deletion queue destroys them. Returns whether the swapchain is
    /// suboptimal.
    pub fn present(&self, present_info: vk::PresentInfoKHR) -> Result<bool, MemoryError> {
        let queue = self.queue();
        self.memory.present(*queue, present_info)
    }

    /// Blocks until the GPU has completed the submission that signals `value`.
    pub fn wait(&self, value: u64) -> Result<(), MemoryError> {
        self.memory.wait_timeline(self.id, value)
    }

    pub(crate) fn memory(&self) -> &MemoryContext {
        &self.memory
    }

    fn queue(&self) -> MutexGuard<'_, vk::Queue> {
        self.queue.lock().expect("the queue lock is poisoned")
    }
}
