use std::collections::VecDeque;
use std::sync::Arc;

use crate::transfer_queue::{RequestId, TransferRequest};
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

    // Moves a waiting request to the back of `priority`'s queue; false when it is no longer waiting.
    pub fn reprioritize(&mut self, id: RequestId, priority: TransferPriority) -> bool {
        let found = self.levels.iter_mut().find_map(|level| {
            let position = level.iter().position(|request| request.id == id)?;
            level.remove(position)
        });
        let Some(mut request) = found else {
            return false;
        };

        request.priority = priority;
        self.push(request);

        true
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use ash::vk;

    use super::*;
    use crate::transfer_queue::{TransferCompletion, TransferWork};

    struct NoCompletion;

    impl TransferCompletion for NoCompletion {
        fn is_cancelled(&self) -> bool {
            false
        }

        fn complete(self: Box<Self>) {}

        fn fail(self: Box<Self>, _: MemoryError) {}
    }

    fn request(value: u64, priority: TransferPriority) -> TransferRequest {
        TransferRequest {
            id: RequestId { value },
            priority,
            work: TransferWork::Readback {
                source: vk::Buffer::null(),
                after: None,
                result: Arc::new(Mutex::new(Vec::new())),
            },
            pieces: VecDeque::new(),
            started: false,
            completion: Box::new(NoCompletion),
        }
    }

    fn order(requests: &mut PendingRequests) -> Vec<u64> {
        std::iter::from_fn(|| requests.pop())
            .map(|request| request.id.value)
            .collect()
    }

    #[test]
    fn a_raised_request_runs_first() {
        let mut requests = PendingRequests::default();
        requests.push(request(0, TransferPriority::Normal));
        requests.push(request(1, TransferPriority::Background));

        assert!(requests.reprioritize(RequestId { value: 1 }, TransferPriority::Critical));
        assert_eq!(order(&mut requests), vec![1, 0]);
    }

    #[test]
    fn a_moved_request_joins_the_back_of_its_level() {
        let mut requests = PendingRequests::default();
        requests.push(request(0, TransferPriority::High));
        requests.push(request(1, TransferPriority::Normal));

        assert!(requests.reprioritize(RequestId { value: 1 }, TransferPriority::High));
        assert_eq!(order(&mut requests), vec![0, 1]);
    }

    #[test]
    fn a_request_no_longer_waiting_is_not_found() {
        let mut requests = PendingRequests::default();

        assert!(!requests.reprioritize(RequestId { value: 7 }, TransferPriority::High));
    }
}
