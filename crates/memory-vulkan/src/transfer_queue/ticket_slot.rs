use std::sync::{Condvar, Mutex, MutexGuard};

use crate::{MemoryError, TransferStatus};

// Where the submission thread leaves a transfer's result for its ticket. `None` once the ticket took it.
pub(crate) struct TicketSlot<T> {
    status: Mutex<Option<TransferStatus<T>>>,
    changed: Condvar,
}

impl<T> TicketSlot<T> {
    pub fn pending() -> TicketSlot<T> {
        TicketSlot {
            status: Mutex::new(Some(TransferStatus::Pending)),
            changed: Condvar::new(),
        }
    }

    pub fn complete(&self, value: T) {
        self.settle(TransferStatus::Complete(value));
    }

    pub fn fail(&self, error: MemoryError) {
        self.settle(TransferStatus::Failed(error));
    }

    // Takes a settled result, leaving a pending one in place.
    pub fn take(&self) -> TransferStatus<T> {
        let mut status = self.status();

        match status.take().expect("a transfer's result is taken once") {
            TransferStatus::Pending => {
                *status = Some(TransferStatus::Pending);
                TransferStatus::Pending
            }
            settled => settled,
        }
    }

    pub fn wait(&self) -> Result<T, MemoryError> {
        let mut status = self.status();

        while matches!(*status, Some(TransferStatus::Pending)) {
            status = self.changed.wait(status).expect("a ticket's lock is poisoned");
        }

        match status.take().expect("a transfer's result is taken once") {
            TransferStatus::Complete(value) => Ok(value),
            TransferStatus::Failed(error) => Err(error),
            TransferStatus::Pending => unreachable!("the loop above waits until the transfer settles"),
        }
    }

    fn settle(&self, settled: TransferStatus<T>) {
        *self.status() = Some(settled);
        self.changed.notify_all();
    }

    fn status(&self) -> MutexGuard<'_, Option<TransferStatus<T>>> {
        self.status.lock().expect("a ticket's lock is poisoned")
    }
}
