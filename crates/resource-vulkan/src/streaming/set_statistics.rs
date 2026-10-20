/// A streamed set's counts: pages in cells, pages wanted but not yet in one, and the bytes its decoded pages hold in
/// RAM. Loads and evictions count since the set was registered.
#[derive(Clone, Copy, Debug, Default)]
pub struct SetStatistics {
    pub resident: usize,
    pub pending: usize,
    pub ram_bytes: u64,
    pub loads: u64,
    pub evictions: u64,
}
