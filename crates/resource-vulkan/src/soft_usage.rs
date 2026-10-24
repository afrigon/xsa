use crate::{EvictionRank, MemoryUsage, SoftOwner};

// Memory eviction may free, and how much keeping it is worth.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SoftUsage {
    pub owner: SoftOwner,
    pub memory: MemoryUsage,
    pub rank: EvictionRank,
}

impl SoftUsage {
    pub fn eviction_order(a: &SoftUsage, b: &SoftUsage) -> std::cmp::Ordering {
        EvictionRank::eviction_order(&a.rank, &b.rank)
    }
}
