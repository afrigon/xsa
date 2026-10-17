use std::sync::Arc;

use crate::transfer_queue::TicketSlot;
use crate::{MemoryError, TransferQueue, TransferStatus};

// A request's result, once the transfer timeline reaches the submission that completes it. It keeps the queue alive,
// so a pending ticket is always settled; dropping it cancels the request.
pub(crate) struct TransferTicket<T> {
    slot: Arc<TicketSlot<T>>,
    _queue: TransferQueue,
}

impl<T> TransferTicket<T> {
    pub fn new(slot: Arc<TicketSlot<T>>, queue: TransferQueue) -> TransferTicket<T> {
        TransferTicket { slot, _queue: queue }
    }

    pub fn poll(&mut self) -> TransferStatus<T> {
        self.slot.take()
    }

    // Waits without giving up the ticket; it must not be polled or waited on again afterwards.
    pub fn wait_settled(&mut self) -> Result<T, MemoryError> {
        self.slot.wait()
    }
}
