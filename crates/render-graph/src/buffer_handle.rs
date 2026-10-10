/// A buffer the graph allocates for the frame being built, created by a pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferHandle {
    pub(crate) index: usize,
}
