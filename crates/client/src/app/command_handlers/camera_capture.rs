use std::fs;
use std::path::PathBuf;

use anyhow::{Context, anyhow, ensure};
use tokio::sync::oneshot;
use xsa_commands::command::{CameraCaptureAction, CameraCaptureCommand};
use xsa_commands::value::{CameraId, Duration, Region};

use crate::app::App;
use crate::app::capture_completion::CaptureCompletion;
use crate::app::capture_end::CaptureEnd;
use crate::app::command_progress::CommandProgress;
use crate::app::command_task::CommandTask;
use crate::app::snapshot::Snapshot;
use crate::app::task_status::TaskStatus;
use crate::app::user_directory::UserDirectory;
use crate::renderer::{CapturedImage, PendingCapture};

const VIDEO_EXTENSION: &str = "mp4";

impl App {
    pub(in crate::app) fn start_capture(&mut self, camera: CameraId, command: CameraCaptureCommand) -> CommandProgress {
        let CameraCaptureCommand {
            output,
            region,
            duration,
            fps,
            action,
        } = command;
        let progress = match (action, duration) {
            (None, None) => self.start_snapshot(output, region),
            (Some(_), Some(_)) => Err(anyhow!("--duration captures a video on its own, without start or stop")),
            (_, _) if region.is_some() => Err(anyhow!("--region only applies to snapshots")),
            (None, Some(duration)) => self.start_timed_capture(camera, duration, fps, output),
            (Some(CameraCaptureAction::Start(start)), None) => App::video_path(start.output)
                .and_then(|path| self.start_video_capture(camera.clone(), start.fps, CaptureEnd::OnStop, path, None))
                .map(|()| CommandProgress::Done(Ok(format!("capturing camera {camera} at {} fps", start.fps)))),
            (Some(CameraCaptureAction::Stop(_)), None) => self
                .stop_video_capture(&camera)
                .map(|receiver| CommandProgress::Running(Box::new(CaptureCompletion { receiver }))),
        };

        progress.unwrap_or_else(|err| CommandProgress::Done(Err(err)))
    }

    fn start_snapshot(&mut self, output: Option<PathBuf>, region: Option<Region>) -> anyhow::Result<CommandProgress> {
        let renderer = self.renderer.as_mut().context("the renderer is not ready")?;

        Ok(CommandProgress::Running(Box::new(SnapTask {
            output,
            region,
            capture: renderer.request_capture(),
        })))
    }

    fn start_timed_capture(
        &mut self,
        camera: CameraId,
        duration: Duration,
        fps: u32,
        output: Option<PathBuf>,
    ) -> anyhow::Result<CommandProgress> {
        let frames = (duration.real_time()?.as_secs_f64() * f64::from(fps)).round() as u64;
        ensure!(frames > 0, "the capture is shorter than one frame");
        let path = App::video_path(output)?;
        let (completion, receiver) = oneshot::channel();
        let end = CaptureEnd::AfterFrames { frames };
        self.start_video_capture(camera, fps, end, path, Some(completion))?;

        Ok(CommandProgress::Running(Box::new(CaptureCompletion { receiver })))
    }

    fn video_path(output: Option<PathBuf>) -> anyhow::Result<PathBuf> {
        let path = match output {
            Some(path) => path,
            None => UserDirectory::Videos.new_file(VIDEO_EXTENSION)?,
        };

        if let Some(directory) = path.parent().filter(|directory| !directory.as_os_str().is_empty()) {
            fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
        }

        Ok(path)
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
