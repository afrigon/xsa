// What a finished read did to its page.
pub(crate) enum ReadOutcome {
    Cancelled,
    Cached,
    Absent,
    Failed(anyhow::Error),
    // The page is gone from its table, or is not reading.
    Ignored,
}
