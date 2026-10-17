use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use ash::vk;

use crate::transfer_queue::{PendingRequests, RequestId, TransferRequest};
use crate::{DeviceObject, MemoryContext, MemoryError, QueueSharing, Timeline, TransferPriority};

// What the queue's handles and its submission thread share.
pub(crate) struct TransferShared {
    pub memory: MemoryContext,
    pub timeline: Timeline,
    pub timeline_semaphore: vk::Semaphore,
    pub family: u32,
    pub sharing: QueueSharing,
    // A timeline semaphore the host signals when it queues a request, so the thread waits for requests and for the
    // GPU at once.
    pub wake: DeviceObject<vk::Semaphore>,
    requests: Mutex<PendingRequests>,
    // The latest transfer timeline value the thread saw completed, raised before it settles any ticket.
    observed: AtomicU64,
    next_request: AtomicU64,
}

impl TransferShared {
    pub fn new(
        memory: &MemoryContext,
        timeline: Timeline,
        family: u32,
        sharing: QueueSharing,
    ) -> Result<TransferShared, MemoryError> {
        let mut type_info = vk::SemaphoreTypeCreateInfo::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);
        let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut type_info);
        let wake = DeviceObject::new(memory, unsafe { memory.device().create_semaphore(&create_info, None) }?);

        Ok(TransferShared {
            memory: memory.clone(),
            timeline_semaphore: timeline.semaphore(),
            timeline,
            family,
            sharing,
            wake,
            requests: Mutex::new(PendingRequests::default()),
            observed: AtomicU64::new(0),
            next_request: AtomicU64::new(0),
        })
    }

    pub fn enqueue(&self, request: TransferRequest) {
        let mut requests = self.requests();

        if let Some(failure) = &requests.failure {
            request.completion.fail(MemoryError::Transfer(failure.clone()));
            return;
        }

        requests.push(request);
        self.wake(&mut requests);
    }

    pub fn next_request_id(&self) -> RequestId {
        RequestId {
            value: self.next_request.fetch_add(1, Ordering::Relaxed),
        }
    }

    // A raised request may now preempt the one being copied, so the thread looks again.
    pub fn reprioritize(&self, id: RequestId, priority: TransferPriority) {
        let mut requests = self.requests();

        if requests.reprioritize(id, priority) {
            self.wake(&mut requests);
        }
    }

    // Signalled under the requests' lock: host signals of a timeline semaphore must increase.
    pub fn wake(&self, requests: &mut PendingRequests) {
        requests.wake_value += 1;
        let signal_info = vk::SemaphoreSignalInfo::default()
            .semaphore(self.wake.handle())
            .value(requests.wake_value);

        if let Err(err) = unsafe { self.memory.device().signal_semaphore(&signal_info) } {
            tracing::error!("waking the transfer thread: {err}");
        }
    }

    pub fn requests(&self) -> MutexGuard<'_, PendingRequests> {
        self.requests.lock().expect("the transfer requests' lock is poisoned")
    }

    pub fn observe(&self, completed: u64) {
        self.observed.fetch_max(completed, Ordering::Release);
    }

    pub fn observed(&self) -> u64 {
        self.observed.load(Ordering::Acquire)
    }

    pub fn completed(&self) -> Result<u64, MemoryError> {
        Ok(unsafe {
            self.memory
                .device()
                .get_semaphore_counter_value(self.timeline_semaphore)
        }?)
    }
}
