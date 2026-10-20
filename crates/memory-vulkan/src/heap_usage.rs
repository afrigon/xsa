/// Bytes the driver lets this process use in a set of memory heaps, and the bytes it uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeapUsage {
    pub budget: u64,
    pub usage: u64,
    /// Bytes of `usage` in the allocator's shared blocks that no object occupies: the driver counts them, yet new
    /// objects in shared memory fill them first.
    pub unallocated: u64,
}
