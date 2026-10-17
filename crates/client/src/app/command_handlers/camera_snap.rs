use std::path::PathBuf;

use xsa_commands::command::CameraSnapCommand;
use xsa_commands::value::Region;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::command_task::CommandTask;
use crate::app::snapshot::Snapshot;
use crate::app::task_status::TaskStatus;
use crate::renderer::{CapturedImage, PendingCapture, Renderer};

impl ClientCommandHandler for CameraSnapCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let settling_frames = app.renderer.as_ref().map_or(0, Renderer::settling_frames);

        CommandProgress::Running(Box::new(SnapTask {
            frames_remaining: settling_frames,
            output: self.output,
            region: self.region,
            capture: None,
        }))
    }
}

// Keeps rendering until temporal effects (exposure metering, anti-aliasing history) reflect the scene, then while the
// captured frame comes back from the GPU.
struct SnapTask {
    frames_remaining: u32,
    output: Option<PathBuf>,
    region: Option<Region>,
    capture: Option<PendingCapture>,
}

impl CommandTask for SnapTask {
    fn poll(&mut self, app: &mut App) -> TaskStatus {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;

            return TaskStatus::Pending;
        }

        let Some(capture) = &mut self.capture else {
            return match app.capture() {
                Ok(capture) => {
                    self.capture = Some(capture);
                    TaskStatus::Pending
                }
                Err(err) => TaskStatus::Done(Err(err)),
            };
        };

        match capture.poll() {
            Ok(None) => TaskStatus::Pending,
            Ok(Some(image)) => TaskStatus::Done(self.save(image)),
            Err(err) => TaskStatus::Done(Err(err)),
        }
    }
}

impl SnapTask {
    fn save(&mut self, image: CapturedImage) -> anyhow::Result<String> {
        let mut snapshot = Snapshot::new(image)?;

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
