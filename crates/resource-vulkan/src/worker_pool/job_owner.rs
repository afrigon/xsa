use crate::EntryId;

// What a job loads: an entry can be reprioritized while it waits, a streamed page is ordered by its set instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JobOwner {
    Entry(EntryId),
    Page,
}
