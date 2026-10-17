use std::collections::VecDeque;
use std::sync::Arc;

use crate::transfer_queue::TransferRequest;
use crate::{MemoryError, TransferPriority};

// Requests waiting for the submission thread, a queue per priority, and what tells the thread to look at them.
#[derive(Default)]
pub(crate) struct PendingRequests {
    levels: [VecDeque<TransferRequest>; TransferPriority::ALL.len()],
    // The value last signalled on the wake semaphore; host signals must increase.
    pub wake_value: u64,
    pub shutdown: bool,
    // Set when the thread stopped on an error, so later requests fail at once.
    pub failure: Option<Arc<MemoryError>>,
}

impl PendingRequests {
    pub fn push(&mut self, request: TransferRequest) {
        self.levels[request.priority.index()].push_back(request);
    }

    pub fn highest_priority(&self) -> Option<TransferPriority> {
        TransferPriority::ALL
            .into_iter()
            .find(|priority| !self.levels[priority.index()].is_empty())
    }

    pub fn pop(&mut self) -> Option<TransferRequest> {
        let priority = self.highest_priority()?;
        self.levels[priority.index()].pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.highest_priority().is_none()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = TransferRequest> + '_ {
        self.levels.iter_mut().flat_map(|level| level.drain(..))
    }
}
