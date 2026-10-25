use std::time::{Duration, Instant};

use xsa_units::TICK_RATE_HERTZ;

use super::window_visibility::WindowVisibility;

// Platforms that never report an occluded window only stop asking for frames, so a window without a frame for this
// long counts as hidden. It must stay well above the frame interval of the slowest display.
const FRAME_TIMEOUT: Duration = Duration::from_millis(500);

// Updates run with every frame the window draws; while it draws none, they run on their own at the tick rate, so
// the connection, commands and tasks keep going in the background.
pub(super) struct UpdateSchedule {
    visibility: WindowVisibility,
    last_frame: Option<Instant>,
    last_update: Option<Instant>,
}

impl UpdateSchedule {
    pub fn new() -> UpdateSchedule {
        UpdateSchedule {
            visibility: WindowVisibility::Visible,
            last_frame: None,
            last_update: None,
        }
    }

    pub fn visibility(&self) -> WindowVisibility {
        self.visibility
    }

    pub fn set_visibility(&mut self, visibility: WindowVisibility) {
        self.visibility = visibility;
    }

    // The seconds since the previous frame, unless the window was not drawing before this one.
    pub fn record_frame(&mut self, now: Instant) -> Option<f64> {
        let delta_seconds = self
            .is_drawing(now)
            .then(|| self.last_frame.map(|last| (now - last).as_secs_f64()));
        self.last_frame = Some(now);

        delta_seconds.flatten()
    }

    pub fn record_update(&mut self, now: Instant) {
        self.last_update = Some(now);
    }

    pub fn is_due(&self, now: Instant) -> bool {
        !self.is_drawing(now) && self.next_update().is_none_or(|next| now >= next)
    }

    pub fn next_wake(&self, now: Instant) -> Instant {
        match self.last_frame {
            Some(last) if self.is_drawing(now) => last + FRAME_TIMEOUT,
            _ => self.next_update().unwrap_or(now),
        }
    }

    fn is_drawing(&self, now: Instant) -> bool {
        self.visibility == WindowVisibility::Visible && self.last_frame.is_some_and(|last| now - last < FRAME_TIMEOUT)
    }

    fn next_update(&self) -> Option<Instant> {
        self.last_update
            .map(|last| last + Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Duration = Duration::from_millis(16);

    fn drawing_schedule(start: Instant) -> UpdateSchedule {
        let mut schedule = UpdateSchedule::new();
        schedule.record_frame(start);
        schedule.record_update(start);

        schedule
    }

    #[test]
    fn a_late_frame_does_not_bring_an_update_of_its_own() {
        let start = Instant::now();
        let schedule = drawing_schedule(start);

        assert!(!schedule.is_due(start + FRAME * 3));
        assert_eq!(schedule.next_wake(start), start + FRAME_TIMEOUT);
    }

    #[test]
    fn updates_run_at_the_tick_rate_once_frames_stop() {
        let start = Instant::now();
        let mut schedule = drawing_schedule(start);
        let hidden = start + FRAME_TIMEOUT;

        assert!(schedule.is_due(hidden));
        schedule.record_update(hidden);
        assert!(!schedule.is_due(hidden + FRAME / 2));
        assert_eq!(
            schedule.next_wake(hidden),
            hidden + Duration::from_secs_f64(1.0 / TICK_RATE_HERTZ)
        );
    }

    #[test]
    fn an_occluded_window_updates_without_waiting_for_the_timeout() {
        let start = Instant::now();
        let mut schedule = drawing_schedule(start);
        schedule.set_visibility(WindowVisibility::Occluded);

        assert!(schedule.is_due(start + FRAME * 2));
    }

    #[test]
    fn the_first_frame_back_has_no_frame_interval() {
        let start = Instant::now();
        let mut schedule = drawing_schedule(start);

        assert_eq!(schedule.record_frame(start + FRAME_TIMEOUT * 4), None);
        assert!(schedule.record_frame(start + FRAME_TIMEOUT * 4 + FRAME).is_some());
    }
}
