use std::hash::Hash;
use std::path::Path;

use crate::{LoadContext, MemoryUsage, PendingResource};

/// A kind of resource file and how to load it. A new kind is a new implementation; `Resources` does not change.
pub trait ResourceKind: Sized + Send + Sync + 'static {
    /// How to read the file, such as a texture's color space. Part of the cache key: the same file loaded with other
    /// options is another resource.
    type Options: Clone + Eq + Hash + Send + Sync + 'static;

    /// On a worker thread: reads and decodes `path`, computes the CPU-side data the resource keeps, creates its GPU
    /// objects and queues their uploads.
    fn load(
        context: &LoadContext,
        path: &Path,
        options: &Self::Options,
    ) -> anyhow::Result<Box<dyn PendingResource<Self>>>;

    /// The memory it holds, counted against the budget once nothing references it.
    fn memory(&self) -> MemoryUsage;
}
