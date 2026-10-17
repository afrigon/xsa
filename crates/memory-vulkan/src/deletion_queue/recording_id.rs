/// Identifies an open recording among its context's recordings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RecordingId {
    pub index: u64,
}
