use anyhow::Context;
use xsa_commands::command::CameraTeleportCommand;

use crate::app::App;
use crate::app::camera_controller::TeleportTarget;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::command_task::CommandTask;
use crate::app::task_status::TaskStatus;

// The renderer states the heights' demand for the new position in the frame after it is placed, and the reload it
// starts shows in the frame after that.
const FRAMES_BEFORE_REQUESTS_SHOW: u64 = 2;

impl ClientCommandHandler for CameraTeleportCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        match TeleportTask::start(&self, app) {
            Ok(task) => CommandProgress::Running(Box::new(task)),
            Err(err) => CommandProgress::Done(Err(err)),
        }
    }
}

// Keeps the camera at the asked altitude above the ground while the heights there load finer, replying once the
// ground is as fine as it will get.
struct TeleportTask {
    target: TeleportTarget,
    reply: String,
    first_frame: u64,
}

impl TeleportTask {
    fn start(command: &CameraTeleportCommand, app: &mut App) -> anyhow::Result<TeleportTask> {
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let cameras = app.cameras.as_mut().context("the cameras are not ready")?;
        let (target, reply) = cameras.teleport(command, world)?;
        let renderer = app.renderer.as_ref().context("the renderer is not ready")?;

        Ok(TeleportTask {
            target,
            reply,
            first_frame: renderer.frames_rendered(),
        })
    }
}

impl CommandTask for TeleportTask {
    fn poll(&mut self, app: &mut App) -> TaskStatus {
        let (Some(world), Some(cameras), Some(renderer)) = (app.world.as_ref(), app.cameras.as_mut(), &app.renderer)
        else {
            return TaskStatus::Done(Err(anyhow::anyhow!("left the simulation while teleporting")));
        };
        if let Err(err) = cameras.stand(self.target, world) {
            return TaskStatus::Done(Err(err));
        }

        let frames = renderer.frames_rendered() - self.first_frame;

        if frames > FRAMES_BEFORE_REQUESTS_SHOW && world.is_ground_settled(self.target.body) {
            return TaskStatus::Done(Ok(std::mem::take(&mut self.reply)));
        }

        TaskStatus::Pending
    }

    fn needs_frames(&self) -> bool {
        true
    }
}
