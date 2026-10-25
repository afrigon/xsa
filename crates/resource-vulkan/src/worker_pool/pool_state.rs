use std::time::Duration;

use crate::worker_pool::JobQueue;

// What the workers wait on.
#[derive(Default)]
pub(crate) struct PoolState {
    pub jobs: JobQueue,
    // Workers whose index is at or above it park.
    pub active_workers: usize,
    pub read_delay: Option<Duration>,
    pub shutdown: bool,
}
