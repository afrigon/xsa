use crate::EntryId;
use crate::streaming::{ChunkId, PageOwner};

// What eviction may free: an unreferenced cached resource, a chunk of streamed pages with every page in it, or a
// streamed page's decoded bytes in RAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoftOwner {
    Entry(EntryId),
    Chunk(ChunkId),
    PageBytes(PageOwner),
}
