use crate::streaming::ChunkId;

// One layer of one chunk, ordered by chunk then layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LayerAddress {
    pub chunk: ChunkId,
    pub layer: u32,
}
