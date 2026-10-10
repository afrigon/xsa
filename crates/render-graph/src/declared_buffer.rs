use crate::{BufferState, GraphBufferDescription};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DeclaredBuffer {
    Transient(GraphBufferDescription),
    Imported {
        import: usize,
        name: &'static str,
        initial: BufferState,
        final_state: BufferState,
    },
}

impl DeclaredBuffer {
    pub fn name(&self) -> &'static str {
        match self {
            DeclaredBuffer::Transient(description) => description.name,
            DeclaredBuffer::Imported { name, .. } => name,
        }
    }

    pub fn persists(&self) -> bool {
        !matches!(self, DeclaredBuffer::Transient(_))
    }
}
