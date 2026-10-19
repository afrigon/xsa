// A set's counts for its periodic statistics line.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SetStatistics {
    pub resident: usize,
    pub pending: usize,
    pub ram_bytes: u64,
    // Since the set was registered.
    pub loads: u64,
    pub evictions: u64,
}
