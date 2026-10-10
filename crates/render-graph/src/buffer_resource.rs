use crate::{BufferHandle, ImportedBufferHandle};

/// A buffer of the frame being built, created by a pass or imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferResource {
    pub(crate) buffer: usize,
}

impl From<BufferHandle> for BufferResource {
    fn from(buffer: BufferHandle) -> BufferResource {
        BufferResource { buffer: buffer.index }
    }
}

impl From<ImportedBufferHandle> for BufferResource {
    fn from(buffer: ImportedBufferHandle) -> BufferResource {
        BufferResource { buffer: buffer.index }
    }
}
