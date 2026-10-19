use crate::streaming::ChunkId;

// One cell of one chunk, ordered by chunk then cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CellAddress {
    pub chunk: ChunkId,
    pub cell: u32,
}
