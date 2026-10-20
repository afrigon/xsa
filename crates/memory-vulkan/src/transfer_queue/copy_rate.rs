use std::time::{Duration, Instant};

// How fast the transfer queue copies, from the last batches seen completing: their bytes over the time the GPU spent
// on them, which starts when they were submitted or when the batch before them completed, whichever is later.
#[derive(Default)]
pub(crate) struct CopyRate {
    bytes_per_second: f64,
    last_completion: Option<Instant>,
}

impl CopyRate {
    pub fn measure(&mut self, bytes: u64, submitted_at: Instant, completed_at: Instant) {
        let started_at = self.last_completion.map_or(submitted_at, |last| last.max(submitted_at));
        let elapsed = completed_at.duration_since(started_at).as_secs_f64();

        if elapsed > 0.0 {
            self.bytes_per_second = bytes as f64 / elapsed;
        }

        self.last_completion = Some(completed_at);
    }

    // The bytes the queue copies in `latency`; nothing until a batch was measured.
    pub fn bytes_in(&self, latency: Duration) -> u64 {
        (self.bytes_per_second * latency.as_secs_f64()) as u64
    }

    // How long the queue takes to copy `bytes`; nothing until a batch was measured.
    pub fn duration_of(&self, bytes: u64) -> Duration {
        if self.bytes_per_second == 0.0 {
            return Duration::ZERO;
        }

        Duration::from_secs_f64(bytes as f64 / self.bytes_per_second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_waiting_behind_the_previous_batch_is_not_copy_time() {
        let start = Instant::now();
        let mut rate = CopyRate::default();
        rate.measure(1000, start, start + Duration::from_secs(1));
        rate.measure(1000, start, start + Duration::from_secs(2));

        assert_eq!(rate.bytes_in(Duration::from_secs(1)), 1000);
    }

    #[test]
    fn nothing_is_budgeted_before_a_measurement() {
        assert_eq!(CopyRate::default().bytes_in(Duration::from_secs(1)), 0);
        assert_eq!(CopyRate::default().duration_of(1000), Duration::ZERO);
    }
}
