use crate::{MemoryUsage, SoftOwner};

// Memory eviction may free, ordered least recently used first, then least important first.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SoftUsage {
    pub owner: SoftOwner,
    pub memory: MemoryUsage,
    // The update it was last used in.
    pub last_use: u64,
    // The score of the streamed pages it holds that are wanted now.
    pub score: f32,
}

impl SoftUsage {
    pub fn eviction_order(a: &SoftUsage, b: &SoftUsage) -> std::cmp::Ordering {
        a.last_use.cmp(&b.last_use).then(a.score.total_cmp(&b.score))
    }
}
