use std::sync::{Arc, Mutex, Weak};

use crate::transfer_queue::{TicketSlot, TransferCompletion};
use crate::{Buffer, MemoryError};

// Keeps the source alive until its copies complete, then hands the bytes they filled to the ticket.
pub(crate) struct ReadbackCompletion {
    _source: Buffer,
    result: Arc<Mutex<Vec<u8>>>,
    slot: Weak<TicketSlot<Vec<u8>>>,
}

impl ReadbackCompletion {
    pub fn new(source: Buffer, result: Arc<Mutex<Vec<u8>>>, slot: Weak<TicketSlot<Vec<u8>>>) -> ReadbackCompletion {
        ReadbackCompletion {
            _source: source,
            result,
            slot,
        }
    }
}

impl TransferCompletion for ReadbackCompletion {
    fn is_cancelled(&self) -> bool {
        self.slot.strong_count() == 0
    }

    fn complete(self: Box<Self>) {
        if let Some(slot) = self.slot.upgrade() {
            let bytes = std::mem::take(&mut *self.result.lock().expect("a readback's lock is poisoned"));
            slot.complete(bytes);
        }
    }

    fn fail(self: Box<Self>, error: MemoryError) {
        if let Some(slot) = self.slot.upgrade() {
            slot.fail(error);
        }
    }
}
