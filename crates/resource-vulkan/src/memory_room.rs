// Memory that can be taken without going over budget. `ram` is `None` when RAM has no budget: the system could not
// be read and no limit is set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MemoryRoom {
    pub video: u64,
    pub ram: Option<u64>,
}
