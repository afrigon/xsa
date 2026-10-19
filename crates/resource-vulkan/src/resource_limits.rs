use std::num::NonZeroUsize;
use std::time::Duration;

/// Caps below what the machine reports, to see how loading behaves on a smaller one. `None` leaves a limit to the
/// machine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceLimits {
    pub max_cpu_cores: Option<NonZeroUsize>,
    /// Bytes of RAM.
    pub max_memory_usage: Option<u64>,
    /// Bytes of video memory.
    pub max_video_memory_usage: Option<u64>,
    /// Added to every read, to see loading and its fallbacks as on a slow disk.
    pub read_delay: Option<Duration>,
}
