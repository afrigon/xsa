use crate::EvictionRank;
use crate::streaming::PageOwner;

// A page whose cell a more important page may take.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Victim {
    pub owner: PageOwner,
    pub rank: EvictionRank,
}
