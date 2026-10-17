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

    pub fn wait(self) -> Result<T, MemoryError> {
        self.slot.wait()
    }
}
