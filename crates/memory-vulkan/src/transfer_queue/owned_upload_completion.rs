use std::sync::Weak;

use crate::MemoryError;
use crate::transfer_queue::{TicketSlot, TransferCompletion};

// Hands a whole destination, kept alive by the request, back to its ticket.
pub(crate) struct OwnedUploadCompletion<T> {
    destination: T,
    slot: Weak<TicketSlot<T>>,
}

impl<T> OwnedUploadCompletion<T> {
    pub fn new(destination: T, slot: Weak<TicketSlot<T>>) -> OwnedUploadCompletion<T> {
        OwnedUploadCompletion { destination, slot }
    }
}

impl<T: Send> TransferCompletion for OwnedUploadCompletion<T> {
    fn is_cancelled(&self) -> bool {
        self.slot.strong_count() == 0
    }

    fn complete(self: Box<Self>) {
        if let Some(slot) = self.slot.upgrade() {
            slot.complete(self.destination);
        }
    }

    fn fail(self: Box<Self>, error: MemoryError) {
        if let Some(slot) = self.slot.upgrade() {
            slot.fail(error);
        }
    }
}
