/// How far a group's loads are, in bytes of their files: a missing or failed file counts as loaded, so progress never
/// moves backwards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupProgress {
    pub loaded_bytes: u64,
    pub total_bytes: u64,
    /// Resources still loading.
    pub loading: usize,
}

impl GroupProgress {
    /// Whether every resource settled: ready, missing or failed.
    pub fn is_loaded(&self) -> bool {
        self.loading == 0
    }
}
