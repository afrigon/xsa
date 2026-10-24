use animation::{CubicBezier, Easing, LimitedMotion};
use xsa_units::{SimulationDuration, SimulationTime, TimeRate, TimeWarp};

use super::MAXIMUM_TIME_RATE;

// Holds still at both ends and rushes through the middle.
const WARP_EASING: Easing = Easing::CubicBezier(CubicBezier::new(0.85, 0.0, 0.15, 1.0));

// A warp under way: time eased from `start` towards the warp's end, never faster than the maximum rate.
pub(crate) struct RunningWarp {
    warp: TimeWarp,
    start: SimulationTime,
    motion: LimitedMotion,
    // Real seconds the warp has run, pauses left out.
    elapsed: f64,
}

impl RunningWarp {
    pub fn new(warp: TimeWarp, start: SimulationTime, seconds: f64) -> RunningWarp {
        let distance = warp.until.seconds_since(start);

        RunningWarp {
            warp,
            start,
            motion: LimitedMotion::new(distance, seconds, WARP_EASING, MAXIMUM_TIME_RATE.multiplier),
            elapsed: 0.0,
        }
    }

    pub fn warp(&self) -> TimeWarp {
        self.warp
    }

    pub fn rate(&self) -> TimeRate {
        TimeRate {
            multiplier: self.motion.speed(self.elapsed),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.motion.is_finished(self.elapsed)
    }

    // Runs the warp for `seconds` of real time, returning the time it reaches.
    pub fn advance(&mut self, seconds: f64) -> SimulationTime {
        self.elapsed += seconds;

        if self.is_finished() {
            return self.warp.until;
        }

        self.start.advanced_by(SimulationDuration {
            seconds: self.motion.position(self.elapsed),
        })
    }

    // Starts over from `now`, as after a pause during which time moved some other way, over the real time it had
    // left; none when time already reached the end.
    pub fn resumed(&self, now: SimulationTime) -> Option<RunningWarp> {
        if now >= self.warp.until {
            return None;
        }

        Some(RunningWarp::new(self.warp, now, self.motion.duration() - self.elapsed))
    }
}
