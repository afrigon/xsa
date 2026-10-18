use crate::JobOwner;

// Loads one entry's file, or reads one streamed page, on a worker.
pub(crate) struct Job {
    pub owner: JobOwner,
    pub run: Box<dyn FnOnce() + Send>,
}
