use crate::EntryId;

// An entry once its last handle dropped. An entry released again has a newer sequence, so an older place in the queue
// is stale.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UnreferencedEntry {
    pub sequence: u64,
    pub entry: EntryId,
}
