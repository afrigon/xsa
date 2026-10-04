const MEASUREMENT_SECONDS: f64 = 0.5;

// Frames per second averaged over fixed windows, so the readout changes slowly enough to read.
pub(super) struct FrameRate {
    frames: u32,
    elapsed_seconds: f64,
    frames_per_second: Option<f64>,
}

impl FrameRate {
    pub fn new() -> FrameRate {
        FrameRate {
            frames: 0,
            elapsed_seconds: 0.0,
            frames_per_second: None,
        }
    }

    pub fn record(&mut self, delta_seconds: f64) {
        self.frames += 1;
        self.elapsed_seconds += delta_seconds;

        if self.elapsed_seconds >= MEASUREMENT_SECONDS {
            self.frames_per_second = Some(f64::from(self.frames) / self.elapsed_seconds);
            self.frames = 0;
            self.elapsed_seconds = 0.0;
        }
    }

    pub fn frames_per_second(&self) -> Option<f64> {
        self.frames_per_second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_over_a_window() {
        let mut rate = FrameRate::new();

        for _ in 0..7 {
            rate.record(1.0 / 16.0);
        }

        assert_eq!(rate.frames_per_second(), None);
        rate.record(1.0 / 16.0);
        assert_eq!(rate.frames_per_second(), Some(16.0));
    }
}
