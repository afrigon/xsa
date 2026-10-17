use crate::transfer_queue::TransferTicket;
use crate::{MemoryError, TransferStatus};

/// A pending readback, yielding the bytes copied back from the GPU. Dropping it cancels the readback if it has not
/// started.
pub struct ReadbackTicket {
    ticket: TransferTicket<Vec<u8>>,
}

impl ReadbackTicket {
    pub(crate) fn new(ticket: TransferTicket<Vec<u8>>) -> ReadbackTicket {
        ReadbackTicket { ticket }
    }

    /// # Panics
    ///
    /// If polled again after it returned a completed or failed readback.
    pub fn poll(&mut self) -> TransferStatus<Vec<u8>> {
        self.ticket.poll()
    }

    /// Blocks until the readback completes.
    pub fn wait(self) -> Result<Vec<u8>, MemoryError> {
        self.ticket.wait()
    }
}
