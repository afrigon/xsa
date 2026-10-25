// Why a frame renders: to be shown, which a running video also captures, or for a snapshot, which it never does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FrameRole {
    Display,
    Snapshot,
}
