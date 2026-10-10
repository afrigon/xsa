use crate::ImageState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ImageBarrierSource {
    Known(ImageState),
    // The image's state when the previous frame ended, read when recording. Discarding skips its layout.
    PreviousFrame { discard: bool },
}
