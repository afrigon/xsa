use crate::memory_context::present_fences::PresentFences;
use crate::{Deletion, DeletionQueue, Frontier, MemoryError, OpenRecordings, RecordingId, TimelineSemaphore};

// What the GPU has been given and what waits for it, kept under one lock so a frontier is read consistently.
#[derive(Default)]
pub(crate) struct Bookkeeping {
    pub deletions: DeletionQueue<Deletion>,
    pub timelines: Vec<TimelineSemaphore>,
    pub presents: PresentFences,
    pub recordings: OpenRecordings<Deletion>,
}

impl Bookkeeping {
    pub fn defer(&mut self, deletion: Deletion) {
        if let Some(deletion) = self.recordings.hold(deletion) {
            let frontier = self.submitted();
            self.deletions.push(&frontier, deletion);
        }
    }

    // A closed recording was submitted, so the frontier includes it, or abandoned, so the GPU never sees it.
    pub fn close_recording(&mut self, id: RecordingId) {
        let released = self.recordings.close(id);
        let frontier = self.submitted();

        for deletion in released {
            self.deletions.push(&frontier, deletion);
        }
    }

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
