use std::fmt::Debug;
use std::hash::Hash;

/// How a client's pages are addressed and read. A streamed set calls it on `Resources`' workers.
pub trait PageSource: Send + Sync + 'static {
    /// Names one page, such as a terrain page's face, level and position.
    type Key: Copy + Eq + Hash + Debug + Send + Sync + 'static;

    /// The page one level coarser that covers `key`, or `None` for the coarsest.
    fn coarser(&self, key: Self::Key) -> Option<Self::Key>;

    /// On a worker: reads and decodes `key` into its set's page format, or `None` when the source has no data for
    /// it, in which case its coarser page stands in.
    fn read(&self, key: Self::Key) -> anyhow::Result<Option<Vec<u8>>>;
}
