use crate::EntryId;
use crate::streaming::{ChunkId, PageOwner};

// What eviction may free: an unreferenced cached resource, a chunk of streamed pages with every page in it, a
// streamed page's decoded bytes in RAM, or the retained levels of a mip-streamed resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoftOwner {
    Entry(EntryId),
    Chunk(ChunkId),
    PageBytes(PageOwner),
    RetainedLevels(EntryId),
}
