use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::streaming::{AnySet, PageFormat, PagePool, SetId, SetStatistics};

const INTERVAL: Duration = Duration::from_secs(1);

// Logs each set's statistics once per interval, under the `streaming` target.
pub(crate) struct StreamingLog {
    last: Instant,
    previous: HashMap<SetId, SetStatistics>,
}

impl StreamingLog {
    pub fn new() -> StreamingLog {
        StreamingLog {
            last: Instant::now(),
            previous: HashMap::new(),
        }
    }

    pub fn update(&mut self, sets: &[Arc<dyn AnySet>], pools: &HashMap<PageFormat, PagePool>) {
        let elapsed = self.last.elapsed();

        if elapsed < INTERVAL || !tracing::enabled!(target: "streaming", tracing::Level::DEBUG) {
            return;
        }

        self.last = Instant::now();
        let seconds = elapsed.as_secs_f64();
        let mut current = HashMap::new();

        for set in sets {
            let statistics = set.statistics();
            let previous = self.previous.get(&set.id()).copied().unwrap_or_default();
            let pool = pools.get(&set.format());
            tracing::debug!(
                target: "streaming",
                "{}: {} resident of {} cells in {} chunks ({} bytes of video memory), {} bytes in RAM, {} pending, \
                 {:.0} loads/s, {:.0} evictions/s",
                set.name(),
                statistics.resident,
                pool.map_or(0, |pool| pool.allocator.capacity()),
                pool.map_or(0, |pool| pool.chunks().len()),
                pool.map_or(0, PagePool::video_bytes),
                statistics.ram_bytes,
                statistics.pending,
                (statistics.loads - previous.loads) as f64 / seconds,
                (statistics.evictions - previous.evictions) as f64 / seconds,
            );
            current.insert(set.id(), statistics);
        }

        self.previous = current;
    }
}
