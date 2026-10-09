/// An imported buffer in the frame being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferHandle {
    pub(crate) index: usize,
}
