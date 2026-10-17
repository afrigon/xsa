/// Bytes the driver lets this process use in a set of memory heaps, and the bytes it uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeapUsage {
    pub budget: u64,
    pub usage: u64,
}
