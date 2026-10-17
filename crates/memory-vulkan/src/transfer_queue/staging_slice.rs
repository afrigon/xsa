// Where a copy's bytes sit in a staging ring.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StagingSlice {
    pub block: usize,
    pub offset: usize,
}
