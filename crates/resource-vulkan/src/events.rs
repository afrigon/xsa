use crate::EntryId;

// What happened to entries since the last update, from any thread.
#[derive(Default)]
pub(crate) struct Events {
    // Entries whose handle count reached zero.
    pub released: Vec<EntryId>,
    // Entries that settled missing or failed, or became required after they did.
    pub unavailable: Vec<EntryId>,
}
