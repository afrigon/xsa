use crate::BufferState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BufferEndState {
    pub buffer: usize,
    pub state: BufferState,
}
