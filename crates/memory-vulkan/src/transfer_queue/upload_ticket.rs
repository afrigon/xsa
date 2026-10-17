use crate::transfer_queue::TransferTicket;
use crate::{MemoryError, TransferStatus};

/// A pending upload. It hands its destination back once written: the GPU may use the destination from the next
/// submission that waits on `TransferQueue::visibility_wait`. Dropping it cancels the upload if it has not started.
pub struct UploadTicket<T> {
    ticket: TransferTicket<T>,
}

impl<T> UploadTicket<T> {
    pub(crate) fn new(ticket: TransferTicket<T>) -> UploadTicket<T> {
        UploadTicket { ticket }
    }

    /// # Panics
    ///
    /// If polled again after it returned a completed or failed upload.
    pub fn poll(&mut self) -> TransferStatus<T> {
        self.ticket.poll()
    }

    /// Blocks until the upload completes.
    pub fn wait(self) -> Result<T, MemoryError> {
        self.ticket.wait()
    }
}
