use std::time::Instant;

use animation::{EasedProgress, Easing};
use anyhow::anyhow;
use xsa_commands::value::Duration;
use xsa_input::{InputButton, InputChord, InputEvent, InputEventKind, InputSource};

use super::App;
use super::command_progress::CommandProgress;
use super::command_task::CommandTask;
use super::glide_motion::GlideMotion;
use super::task_status::TaskStatus;

// Holds buttons while it injects a motion over a duration. Each update injects the eased progress made since the last
// one, so the shares add up to the whole motion at any frame rate.
pub(super) struct InputGlide {
    motion: GlideMotion,
    held: Vec<InputButton>,
    progress: EasedProgress,
    applied: f64,
    last_poll: Instant,
    resets: u64,
    summary: String,
}

impl InputGlide {
    // Without a duration the whole glide happens before this returns.
    pub fn start(
        app: &mut App,
        motion: GlideMotion,
        held: Vec<InputButton>,
        duration: Option<Duration>,
        easing: Easing,
        summary: String,
    ) -> CommandProgress {
        let seconds = match duration.map(Duration::real_time).transpose() {
            Ok(duration) => duration.map_or(0.0, |duration| duration.as_secs_f64()),
            Err(err) => return CommandProgress::Done(Err(err)),
        };

        if let Err(err) = app.inject(held.iter().map(|button| InputEventKind::Pressed { button: *button })) {
            return CommandProgress::Done(Err(err));
        }

        let mut glide = InputGlide {
            motion,
            held,
            progress: EasedProgress::new(seconds, easing),
            applied: 0.0,
            last_poll: Instant::now(),
            resets: app.input_resets,
            summary,
        };

        match glide.poll(app) {
            TaskStatus::Pending => CommandProgress::Running(Box::new(glide)),
            TaskStatus::Done(result) => CommandProgress::Done(result),
        }
    }

    pub fn summary(text: String, hold: Option<&InputChord>) -> String {
        match hold {
            Some(chord) => format!("{text} holding {}", chord.name()),
            None => text,
        }
    }

    // Releases pass even with command input switched off, so a glide never leaves a button held.
    fn release(&self, app: &mut App) {
        for button in self.held.iter().rev() {
            app.handle_input(InputEvent {
                source: InputSource::Command,
                kind: InputEventKind::Released { button: *button },
            });
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
            self.release(app);

            return TaskStatus::Done(Err(err));
        }

        if !self.progress.is_finished() {
            return TaskStatus::Pending;
        }

        self.release(app);

        TaskStatus::Done(Ok(std::mem::take(&mut self.summary)))
    }
}
