use tokio::sync::oneshot;
use xsa_proto::frame::RequestId;
use xsa_units::TimeRate;

use super::capture_end::CaptureEnd;
use super::capture_phase::CapturePhase;

// A camera's capture over time: it holds the simulation's time and steps it by one frame's worth after each frame.
pub(super) struct VideoCapture {
    pub fps: u32,
    pub frames: u64,
    pub end: CaptureEnd,
    pub phase: CapturePhase,
    pub awaiting: Option<RequestId>,
    pub previous_override: Option<TimeRate>,
    pub completion: Option<oneshot::Sender<anyhow::Result<String>>>,
}

impl VideoCapture {
    pub fn is_done(&self) -> bool {
        match self.end {
            CaptureEnd::AfterFrames { frames } => self.frames >= frames,
            CaptureEnd::OnStop => false,
        }
    }

    pub fn complete(&mut self, result: anyhow::Result<String>) {
        if let Some(completion) = self.completion.take() {
            let _ = completion.send(result);
        }
    }
}
