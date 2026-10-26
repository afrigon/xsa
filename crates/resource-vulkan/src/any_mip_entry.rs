use std::sync::{Arc, Mutex};
use std::time::Instant;

use ash::vk;

use crate::{AnyEntry, LoadEnvironment, SoftUsage, WorkerPool};

// A mip-streamed entry of any kind, as the bookkeeping sees it.
pub(crate) trait AnyMipEntry: AnyEntry {
    // Starts loading the levels its holders' demand needs, once no load is under way: finer levels at once, coarser
    // ones once the finer levels have gone unwanted for the demand's retention.
    fn update_residency(
        self: Arc<Self>,
        update: u64,
        now: Instant,
        environment: &Arc<LoadEnvironment>,
        uploading: &Arc<Mutex<Vec<Arc<dyn AnyEntry>>>>,
        pool: &WorkerPool,
    );

    // Records the copy of the levels a finished reload shares with the current value, then makes the reload current.
    fn record_copies(&self, device: &ash::Device, command_buffer: vk::CommandBuffer);

    // The memory its retained levels hold: finer than its holders want, kept for their retention.
    fn retained_usage(&self) -> Option<SoftUsage>;

    // Drops its retained levels at the next update, whatever their retention.
    fn release_retained(&self);
}
