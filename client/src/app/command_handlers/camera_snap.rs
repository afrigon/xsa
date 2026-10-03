use std::path::PathBuf;
use std::time::{Duration, Instant};

use xsa_commands::command::CameraSnapCommand;
use xsa_commands::value::Region;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::command_task::CommandTask;
use crate::app::snapshot::Snapshot;
use crate::app::task_status::TaskStatus;

impl ClientCommandHandler for CameraSnapCommand {
    fn start(self, _app: &mut App) -> CommandProgress {
        let delay_seconds = self.delay.map_or(0.0, |delay| delay.duration().seconds);

        if !(delay_seconds.is_finite() && delay_seconds >= 0.0) {
            return CommandProgress::Done(Err(anyhow::anyhow!("the delay must be a positive duration")));
        }

        CommandProgress::Running(Box::new(SnapTask {
            due: Instant::now() + Duration::from_secs_f64(delay_seconds),
            output: self.output,
            region: self.region,
        }))
    }
}

// Waits in real time while the game keeps rendering, so exposure and other animations progress.
struct SnapTask {
    due: Instant,
    output: Option<PathBuf>,
    region: Option<Region>,
}

impl CommandTask for SnapTask {
    fn poll(&mut self, app: &mut App) -> TaskStatus {
        if Instant::now() < self.due {
            return TaskStatus::Pending;
        }

        TaskStatus::Done(self.capture(app))
    }
}

impl SnapTask {
    fn capture(&mut self, app: &mut App) -> anyhow::Result<String> {
        let mut snapshot = Snapshot::new(app.capture()?)?;

        if let Some(region) = self.region {
            snapshot = snapshot.crop(region)?;
        }

        let path = match self.output.take() {
            Some(path) => path,
            None => Snapshot::default_path()?,
        };

        snapshot.save(&path)
    }
}
