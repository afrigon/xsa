use crate::EntryId;

// Loads one entry's file on a worker; the entry can be reprioritized while its job waits.
pub(crate) struct Job {
    pub entry: EntryId,
    pub run: Box<dyn FnOnce() + Send>,
}
