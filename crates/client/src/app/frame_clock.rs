use std::time::{Duration, Instant};

use super::frame_pace::FramePace;

// The moment each update represents. It advances once per update and never jumps, so switching pace keeps every
// deadline and duration measured against it consistent.
pub(super) struct FrameClock {
    now: Instant,
    wall: Instant,
    pace: FramePace,
}

impl FrameClock {
    pub fn new() -> FrameClock {
        let now = Instant::now();

        FrameClock {
            now,
            wall: now,
            pace: FramePace::Wall,
        }
    }

    pub fn now(&self) -> Instant {
        self.now
    }

    pub fn set_pace(&mut self, pace: FramePace) {
        self.pace = pace;
    }

    // Advances to the next update's moment, returning the time it covers.
    pub fn tick(&mut self) -> Duration {
        let wall = Instant::now();
        let covered = match self.pace {
            FramePace::Wall => wall - self.wall,
            FramePace::Fixed { step } => step,
        };
        self.now += covered;
        self.wall = wall;

        covered
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn a_fixed_pace_advances_exactly_one_step_per_tick() {
        let mut clock = FrameClock::new();
        let start = clock.now();
        let step = Duration::from_millis(16);
        clock.set_pace(FramePace::Fixed { step });
        clock.tick();
        clock.tick();

        assert_eq!(clock.now() - start, step * 2);
    }
}
