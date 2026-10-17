// Names a resource entry for as long as it is registered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EntryId {
    pub value: u64,
}
