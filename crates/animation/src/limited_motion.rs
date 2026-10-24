use crate::Easing;

/// Covers a distance along an ease-in-out curve over a duration without exceeding a maximum speed.
///
/// Below the maximum the position follows the easing over the whole duration. Above it the curve is split at its
/// midpoint, where a symmetric ease-in-out is steepest: each half becomes a ramp lasting half the duration, scaled to
/// end at the maximum speed, and a cruise at that speed covers the rest. The motion then lasts longer than asked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimitedMotion {
    distance: f64,
    easing: Easing,
    ramp_duration: f64,
    ramp_distance: f64,
    cruise_duration: f64,
    cruise_speed: f64,
}

impl LimitedMotion {
    /// A motion over `distance` meant to last `duration`; both and `max_speed` are above zero.
    pub fn new(distance: f64, duration: f64, easing: Easing, max_speed: f64) -> LimitedMotion {
        let peak_slope = easing.derivative(0.5);
        let ramp_distance = distance.min(duration * max_speed / peak_slope);

        LimitedMotion {
            distance,
            easing,
            ramp_duration: duration / 2.0,
            ramp_distance,
            cruise_duration: (distance - ramp_distance) / max_speed,
            cruise_speed: max_speed,
        }
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn duration(&self) -> f64 {
        2.0 * self.ramp_duration + self.cruise_duration
    }

    pub fn is_finished(&self, elapsed: f64) -> bool {
        elapsed >= self.duration()
    }

    /// The distance covered after `elapsed`.
    pub fn position(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 {
            return 0.0;
        }

        if self.is_finished(elapsed) {
            return self.distance;
        }

        let midpoint = self.easing.apply(0.5);
        let cruise_end = self.ramp_duration + self.cruise_duration;

        if elapsed < self.ramp_duration {
            return self.ramp_distance * self.easing.apply(self.ramp_progress(elapsed));
        }

        let cruised = self.cruise_speed * (elapsed.min(cruise_end) - self.ramp_duration);

        if elapsed < cruise_end {
            return self.ramp_distance * midpoint + cruised;
        }

        let descent = self.easing.apply(self.ramp_progress(elapsed - self.cruise_duration)) - midpoint;

        self.ramp_distance * midpoint + cruised + self.ramp_distance * descent
    }

    /// The rate the distance is covered at after `elapsed`.
    pub fn speed(&self, elapsed: f64) -> f64 {
        if elapsed < 0.0 || self.is_finished(elapsed) {
            return 0.0;
        }

        let cruise_end = self.ramp_duration + self.cruise_duration;
        let ramp_scale = self.ramp_distance / (2.0 * self.ramp_duration);

        if elapsed < self.ramp_duration {
            return ramp_scale * self.easing.derivative(self.ramp_progress(elapsed));
        }

        if elapsed < cruise_end {
            return self.cruise_speed;
        }

        ramp_scale
            * self
                .easing
                .derivative(self.ramp_progress(elapsed - self.cruise_duration))
    }

    // The two ramps together are the easing over 0..1, the cruise excluded.
    fn ramp_progress(&self, ramp_elapsed: f64) -> f64 {
        ramp_elapsed / (2.0 * self.ramp_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CubicBezier;

    const EASING: Easing = Easing::CubicBezier(CubicBezier::new(0.85, 0.0, 0.15, 1.0));
    const TOLERANCE: f64 = 1e-6;

    #[test]
    fn below_the_maximum_it_follows_the_easing() {
        let motion = LimitedMotion::new(100.0, 4.0, EASING, 1e9);

        assert_eq!(motion.duration(), 4.0);
        assert!((motion.position(2.0) - 50.0).abs() < TOLERANCE);
        assert!((motion.position(1.0) - 100.0 * EASING.apply(0.25)).abs() < TOLERANCE);
        assert_eq!(motion.position(4.0), 100.0);
    }

    #[test]
    fn above_the_maximum_it_cruises_at_it() {
        let max_speed = 10.0;
        let motion = LimitedMotion::new(1000.0, 4.0, EASING, max_speed);

        assert!(motion.duration() > 4.0);
        assert!((motion.speed(2.0) - max_speed).abs() < TOLERANCE);
        assert!((motion.speed(motion.duration() / 2.0) - max_speed).abs() < TOLERANCE);
        assert_eq!(motion.position(motion.duration()), 1000.0);
    }

    #[test]
    fn the_speed_never_exceeds_the_maximum_and_the_position_is_continuous() {
        let max_speed = 10.0;
        let motion = LimitedMotion::new(1000.0, 4.0, EASING, max_speed);
        let steps = 10_000;
        let step = motion.duration() / steps as f64;
        let mut previous = 0.0;

        for index in 1..=steps {
            let elapsed = index as f64 * step;
            let position = motion.position(elapsed);

            assert!(motion.speed(elapsed) <= max_speed + TOLERANCE);
            assert!(position >= previous);
            assert!(position - previous <= max_speed * step + TOLERANCE);

            previous = position;
        }
    }
}
