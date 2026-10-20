use crate::streaming::{PageFormat, PageOwner};

// Releases `chunks` chunks of a pool: its `evicted` pages give up their cells, and the pages left in the chunks
// released move into the cells freed.
#[derive(Clone, Debug)]
pub(crate) struct PoolShrink {
    pub format: PageFormat,
    pub evicted: Vec<PageOwner>,
    pub chunks: usize,
}
