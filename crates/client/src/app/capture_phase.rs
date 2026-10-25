#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CapturePhase {
    // Waiting for the override that holds time while frames are captured.
    Starting,
    Running,
    // Waiting for the override in place before the capture to come back.
    Finishing,
}
