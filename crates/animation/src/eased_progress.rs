use crate::Easing;

/// Time elapsed over a duration, read through an easing. A finished progress reads 1, so a zero duration is finished
/// at once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EasedProgress {
    elapsed: f64,
    duration: f64,
    easing: Easing,
}

impl EasedProgress {
    pub fn new(duration: f64, easing: Easing) -> EasedProgress {
        EasedProgress {
            elapsed: 0.0,
            duration,
            easing,
        }
    }

    pub fn advance(&mut self, seconds: f64) {
        self.elapsed += seconds;
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// The eased progress in 0..1.
    pub fn value(&self) -> f64 {
        if self.is_finished() {
            return 1.0;
        }

        self.easing.apply(self.elapsed / self.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_duration_is_finished_at_once() {
        let progress = EasedProgress::new(0.0, Easing::Smoothstep);

        assert!(progress.is_finished());
        assert_eq!(progress.value(), 1.0);
    }

    #[test]
    fn progress_follows_its_easing() {
        let mut progress = EasedProgress::new(2.0, Easing::Linear);
        progress.advance(0.5);

        assert_eq!(progress.value(), 0.25);
        assert!(!progress.is_finished());

        progress.advance(1.5);

        assert!(progress.is_finished());
    }
}
