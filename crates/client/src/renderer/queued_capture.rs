use tokio::sync::oneshot::Sender;

use super::capture_readback::CaptureReadback;

// A capture waiting for temporal effects to reflect the scene; its readback, or why it could not start, goes to
// `sender`.
pub(super) struct QueuedCapture {
    pub settling_frames: u32,
    pub sender: Sender<anyhow::Result<CaptureReadback>>,
}
