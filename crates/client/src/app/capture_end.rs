#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CaptureEnd {
    AfterFrames { frames: u64 },
    OnStop,
}
