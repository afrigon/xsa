use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::error::TryRecvError;

use super::CapturedImage;
use super::capture_readback::CaptureReadback;

// A capture the renderer takes once temporal effects settle, then copies back from the GPU.
pub struct PendingCapture {
    state: CaptureState,
}

enum CaptureState {
    Settling {
        readback: Receiver<anyhow::Result<CaptureReadback>>,
    },
    Reading(CaptureReadback),
}

impl PendingCapture {
    pub(super) fn new(readback: Receiver<anyhow::Result<CaptureReadback>>) -> PendingCapture {
        PendingCapture {
            state: CaptureState::Settling { readback },
        }
    }

    // The image once its pixels are back on the CPU.
    pub fn poll(&mut self) -> anyhow::Result<Option<CapturedImage>> {
        if let CaptureState::Settling { readback } = &mut self.state {
            match readback.try_recv() {
                Ok(started) => self.state = CaptureState::Reading(started?),
                Err(TryRecvError::Empty) => return Ok(None),
                Err(TryRecvError::Closed) => anyhow::bail!("the renderer stopped before capturing"),
            }
        }

        match &mut self.state {
            CaptureState::Settling { .. } => Ok(None),
            CaptureState::Reading(readback) => readback.poll(),
        }
    }
}
