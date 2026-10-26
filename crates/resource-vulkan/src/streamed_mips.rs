use std::hash::Hash;
use std::path::Path;

use memory_vulkan::Image;

use crate::{LoadContext, MemoryUsage, PendingResource};

/// A resource whose image holds only the mip levels from a first level down to the smallest, reloaded as its
/// holders' demand changes. `Resources` copies the levels a reload shares with the current value on the GPU, so a
/// reload reads and uploads only the levels the current value lacks.
pub trait StreamedMips: Sized + Send + Sync + 'static {
    /// How to read the file. Part of the cache key, as for a `ResourceKind`.
    type Options: Clone + Eq + Hash + Send + Sync + 'static;

    /// On a worker thread: creates the image of the levels from `first_level` (clamped to the chain), with
    /// `TRANSFER_SRC` and `TRANSFER_DST` usage, and queues the upload of the levels `current` does not hold. Once the
    /// upload completes, the image is in `SHADER_READ_ONLY_OPTIMAL` layout in every level, the shared ones included,
    /// which `Resources` then fills from `current`.
    fn load(
        context: &LoadContext,
        path: &Path,
        options: &Self::Options,
        first_level: u32,
        current: Option<&Self>,
    ) -> anyhow::Result<Box<dyn PendingResource<Self>>>;

    /// The image whose levels stream; its level 0 is the file's `first_level`.
    fn image(&self) -> &Image;

    /// The finest level of the file the image holds.
    fn first_level(&self) -> u32;

    /// The levels of the file's whole chain.
    fn mip_levels(&self) -> u32;

    fn memory(&self) -> MemoryUsage;
}
