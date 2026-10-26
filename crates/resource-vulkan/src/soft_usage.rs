use crate::{MemoryUsage, SoftOwner};

// Memory eviction may free; the least recently used goes first.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SoftUsage {
    pub owner: SoftOwner,
    pub memory: MemoryUsage,
    // The update it was last used in.
    pub last_use: u64,
}
