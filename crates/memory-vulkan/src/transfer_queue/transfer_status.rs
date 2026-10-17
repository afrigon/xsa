use crate::MemoryError;

/// Where a transfer stands when its ticket is polled.
#[derive(Debug)]
pub enum TransferStatus<T> {
    Pending,
    Complete(T),
    Failed(MemoryError),
}
