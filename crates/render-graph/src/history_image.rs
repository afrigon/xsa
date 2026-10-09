use crate::ImageHandle;

/// This frame's view of a history: `current` is written this frame, `previous` holds what the last frame wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryImage {
    pub current: ImageHandle,
    pub previous: ImageHandle,
}
