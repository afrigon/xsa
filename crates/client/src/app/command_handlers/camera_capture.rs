use std::path::PathBuf;

use anyhow::anyhow;
use xsa_commands::command::CameraCaptureCommand;
use xsa_commands::value::Region;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::command_task::CommandTask;
use crate::app::snapshot::Snapshot;
use crate::app::task_status::TaskStatus;
use crate::renderer::{CapturedImage, PendingCapture};

impl ClientCommandHandler for CameraCaptureCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        if self.action.is_some() || self.duration.is_some() {
            return CommandProgress::Done(Err(anyhow!("video captures are not available yet")));
        }

        let Some(renderer) = &mut app.renderer else {
            return CommandProgress::Done(Err(anyhow!("the renderer is not ready")));
        };

        CommandProgress::Running(Box::new(SnapTask {
            output: self.output,
            region: self.region,
            capture: renderer.request_capture(),
        }))
    }
}

struct SnapTask {
    output: Option<PathBuf>,
    region: Option<Region>,
    capture: PendingCapture,
}

impl CommandTask for SnapTask {
    fn poll(&mut self, _app: &mut App) -> TaskStatus {
        match self.capture.poll() {
            Ok(None) => TaskStatus::Pending,
            Ok(Some(image)) => TaskStatus::Done(self.save(image)),
            Err(err) => TaskStatus::Done(Err(err)),
        }
    }

    fn needs_frames(&self) -> bool {
        true
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
