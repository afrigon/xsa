use std::sync::Arc;

// What a read job reports for its page.
pub(crate) enum ReadResult {
    // It was cancelled before it started.
    Cancelled,
    Bytes(Arc<[u8]>),
    // The source has no data for the page.
    Absent,
    Failed(anyhow::Error),
}
