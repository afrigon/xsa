use crate::BufferState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DeclaredBuffer {
    pub name: &'static str,
    pub initial: BufferState,
    pub final_state: BufferState,
}
