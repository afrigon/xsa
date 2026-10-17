use crate::EntryId;

// Loads one entry's file on a worker.
pub(crate) struct Job {
    pub entry: EntryId,
    pub run: Box<dyn FnOnce() + Send>,
}
