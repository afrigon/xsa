// Names a chunk of any pool, never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ChunkId {
    pub value: u64,
}
