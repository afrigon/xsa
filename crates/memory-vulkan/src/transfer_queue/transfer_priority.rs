/// How urgently a transfer is needed. Requests run in this order, and in the order they were made within a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransferPriority {
    Critical,
    High,
    Normal,
    Background,
}

impl TransferPriority {
    pub(crate) const ALL: [TransferPriority; 4] = [
        TransferPriority::Critical,
        TransferPriority::High,
        TransferPriority::Normal,
        TransferPriority::Background,
    ];

    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
