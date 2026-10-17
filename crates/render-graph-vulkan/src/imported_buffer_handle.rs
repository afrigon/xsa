/// An imported buffer in the frame being built. Unlike a `BufferHandle`, it has no device address from the graph: the
/// graph does not own it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedBufferHandle {
    pub(crate) index: usize,
}
