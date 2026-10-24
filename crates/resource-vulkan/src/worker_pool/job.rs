use std::time::Instant;

use crate::JobOwner;

// Loads one entry's file, or reads one streamed page, on a worker. It runs given the moment a worker took it, so its
// read time counts every delay a slow disk adds.
pub(crate) struct Job {
    pub owner: JobOwner,
    pub run: Box<dyn FnOnce(Instant) + Send>,
}
