use std::time::Instant;

use animation::{EasedProgress, Easing};
use anyhow::anyhow;
use xsa_commands::value::Duration;

use super::App;
use super::command_progress::CommandProgress;
use super::command_task::CommandTask;
use super::glide_motion::GlideMotion;
use super::task_status::TaskStatus;

// Injects a motion over a duration. Each update injects the eased progress made since the last one, so the shares add
// up to the whole motion at any frame rate.
pub(super) struct InputGlide {
    motion: GlideMotion,
    progress: EasedProgress,
    applied: f64,
    last_poll: Instant,
    resets: u64,
    summary: String,
}

impl InputGlide {
    pub fn start(
        app: &mut App,
        motion: GlideMotion,
        duration: Duration,
        easing: Easing,
        summary: String,
    ) -> CommandProgress {
        let started = duration
            .real_time()
            .and_then(|duration| app.inject([]).map(|()| duration.as_secs_f64()));

        match started {
            Ok(seconds) => CommandProgress::Running(Box::new(InputGlide {
                motion,
                progress: EasedProgress::new(seconds, easing),
                applied: 0.0,
                last_poll: Instant::now(),
                resets: app.input_resets,
                summary,
            })),
            Err(err) => CommandProgress::Done(Err(err)),
        }
    }
}

impl CommandTask for InputGlide {
    fn poll(&mut self, app: &mut App) -> TaskStatus {
        if app.input_resets != self.resets {
            return TaskStatus::Done(Err(anyhow!("stopped by input reset")));
        }

        let now = Instant::now();
        self.progress.advance((now - self.last_poll).as_secs_f64());
        self.last_poll = now;
        let value = self.progress.value();
        let share = value - self.applied;
        self.applied = value;

        if let Err(err) = self.motion.inject(app, share) {
            return TaskStatus::Done(Err(err));
        }

        if self.progress.is_finished() {
            TaskStatus::Done(Ok(std::mem::take(&mut self.summary)))
        } else {
            TaskStatus::Pending
        }
    }
}
