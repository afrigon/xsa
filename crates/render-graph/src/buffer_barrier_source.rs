use crate::BufferState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BufferBarrierSource {
    Known(BufferState),
    // The buffer's state when the previous frame ended, read when recording.
    PreviousFrame,
}
