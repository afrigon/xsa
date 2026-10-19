use crate::streaming::{CellAddress, PageId, PageRetention};

// A page holding a cell, as eviction sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CellUse {
    pub page: PageId,
    pub cell: CellAddress,
    pub last_use: u64,
    // Its score while it is wanted.
    pub score: Option<f32>,
    pub retention: PageRetention,
}
