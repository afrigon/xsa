use crate::memory_context::present_fences::PresentFences;
use crate::{Deletion, DeletionQueue, Frontier, MemoryError, TimelineSemaphore};

// What the GPU has been given and what waits for it, kept under one lock so a frontier is read consistently.
#[derive(Default)]
pub(crate) struct Bookkeeping {
    pub deletions: DeletionQueue<Deletion>,
    pub timelines: Vec<TimelineSemaphore>,
    pub presents: PresentFences,
}

impl Bookkeeping {
    pub fn submitted(&self) -> Frontier {
        Frontier {
            timelines: self.timelines.iter().map(|timeline| timeline.submitted).collect(),
            presents: self.presents.queued(),
        }
    }

    pub fn completed(&mut self, device: &ash::Device) -> Result<Frontier, MemoryError> {
        let timelines = self
            .timelines
            .iter()
            .map(|timeline| timeline.completed(device))
            .collect::<Result<_, _>>()?;
        let presents = self.presents.completed(device)?;

        Ok(Frontier { timelines, presents })
    }
}
