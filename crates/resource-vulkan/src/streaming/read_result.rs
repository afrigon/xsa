use std::sync::Arc;
use std::time::Duration;

// What a read job reports for its page.
pub(crate) enum ReadResult {
    // It was cancelled before it started.
    Cancelled,
    Bytes { bytes: Arc<[u8]>, read_time: Duration },
    // The source has no data for the page.
    Absent,
    Failed(anyhow::Error),
}
