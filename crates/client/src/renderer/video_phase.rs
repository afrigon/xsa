#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoPhase {
    // Every displayed frame is captured.
    Recording,
    // No frame is captured any more; the frames still encoding are written, then the file closes.
    Closing,
}
