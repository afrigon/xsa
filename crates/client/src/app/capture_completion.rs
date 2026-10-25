use anyhow::anyhow;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

use super::App;
use super::command_task::CommandTask;
use super::task_status::TaskStatus;

// Replies once a video capture finishes.
pub(super) struct CaptureCompletion {
    pub receiver: oneshot::Receiver<anyhow::Result<String>>,
}

impl CommandTask for CaptureCompletion {
    fn poll(&mut self, _app: &mut App) -> TaskStatus {
        match self.receiver.try_recv() {
            Ok(result) => TaskStatus::Done(result),
            Err(TryRecvError::Empty) => TaskStatus::Pending,
            Err(TryRecvError::Closed) => TaskStatus::Done(Err(anyhow!("the capture ended without finishing"))),
        }
    }
}
