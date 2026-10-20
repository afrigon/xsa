use std::time::Duration;

use crate::streaming::{CellAddress, CellContent, PageId, PageRetention};

// A page holding a cell, as eviction sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CellUse {
    pub page: PageId,
    // The cell shaders read it from, or it is copying into.
    pub cell: CellAddress,
    pub content: CellContent,
    pub last_use: u64,
    // Its score while it is wanted.
    pub score: Option<f32>,
    pub retention: PageRetention,
    pub read_time: Duration,
    // Its bytes are in RAM, so losing the cell costs an upload only.
    pub cached: bool,
}
