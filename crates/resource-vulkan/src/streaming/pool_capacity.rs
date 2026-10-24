use crate::streaming::PageFormat;

// A pool as eviction sees it: video memory comes back a whole chunk at a time, once a chunk's worth of its cells are
// free or evicted.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PoolCapacity {
    pub format: PageFormat,
    pub cells_per_chunk: usize,
    pub chunk_size: u64,
    // Cells no page holds, including those returning once the GPU is done with them.
    pub free_cells: usize,
}
