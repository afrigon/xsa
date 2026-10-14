const MEASUREMENT_SECONDS: f64 = 0.5;

// Frames per second and GPU time averaged over fixed windows, so the readouts change slowly enough to read.
pub(super) struct FrameTiming {
    frames: u32,
    elapsed_seconds: f64,
    gpu_milliseconds_sum: f64,
    gpu_samples: u32,
    frames_per_second: Option<f64>,
    gpu_milliseconds: Option<f64>,
}

impl FrameTiming {
    pub fn new() -> FrameTiming {
        FrameTiming {
            frames: 0,
            elapsed_seconds: 0.0,
            gpu_milliseconds_sum: 0.0,
            gpu_samples: 0,
            frames_per_second: None,
            gpu_milliseconds: None,
        }
    }

    // `gpu_milliseconds` is the GPU time of the latest measured frame, when there is one.
    pub fn record(&mut self, delta_seconds: f64, gpu_milliseconds: Option<f32>) {
        self.frames += 1;
        self.elapsed_seconds += delta_seconds;

        if let Some(milliseconds) = gpu_milliseconds {
            self.gpu_milliseconds_sum += f64::from(milliseconds);
            self.gpu_samples += 1;
        }

        if self.elapsed_seconds >= MEASUREMENT_SECONDS {
            self.frames_per_second = Some(f64::from(self.frames) / self.elapsed_seconds);
            self.gpu_milliseconds = (self.gpu_samples > 0)
                .then(|| self.gpu_milliseconds_sum / f64::from(self.gpu_samples))
                .or(self.gpu_milliseconds);
            self.frames = 0;
            self.elapsed_seconds = 0.0;
            self.gpu_milliseconds_sum = 0.0;
            self.gpu_samples = 0;
        }
    }

    pub fn frames_per_second(&self) -> Option<f64> {
        self.frames_per_second
    }

    pub fn gpu_milliseconds(&self) -> Option<f64> {
        self.gpu_milliseconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_SECONDS: f64 = 1.0 / 16.0;

    #[test]
    fn averages_over_a_window() {
        let mut timing = FrameTiming::new();

        for frame in 0..7 {
            timing.record(FRAME_SECONDS, Some(if frame % 2 == 0 { 1.0 } else { 3.0 }));
        }

        assert_eq!(timing.frames_per_second(), None);
        assert_eq!(timing.gpu_milliseconds(), None);
        timing.record(FRAME_SECONDS, Some(3.0));
        assert_eq!(timing.frames_per_second(), Some(16.0));
        assert_eq!(timing.gpu_milliseconds(), Some(2.0));
    }

    #[test]
    fn keeps_the_gpu_time_through_a_window_without_measurements() {
        let mut timing = FrameTiming::new();

        for _ in 0..8 {
            timing.record(FRAME_SECONDS, Some(2.0));
        }

        for _ in 0..8 {
            timing.record(FRAME_SECONDS, None);
        }

        assert_eq!(timing.gpu_milliseconds(), Some(2.0));
    }
}
