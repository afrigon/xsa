use ash::vk;

use crate::{MemoryError, Timeline};

/// A point on a timeline: the value a submission signals once it completes.
#[derive(Clone)]
pub struct TimelineValue {
    pub timeline: Timeline,
    pub value: u64,
}

impl TimelineValue {
    /// A wait for this point, for a submission to another queue that must start `stage` only once it is reached.
    pub fn wait_info(&self, stage: vk::PipelineStageFlags2) -> vk::SemaphoreSubmitInfo<'static> {
        vk::SemaphoreSubmitInfo::default()
            .semaphore(self.timeline.semaphore())
            .value(self.value)
            .stage_mask(stage)
    }

    /// Whether the GPU has completed the submission that signals this point, without waiting for it.
    pub fn is_complete(&self) -> Result<bool, MemoryError> {
        Ok(self.timeline.completed()? >= self.value)
    }
}
