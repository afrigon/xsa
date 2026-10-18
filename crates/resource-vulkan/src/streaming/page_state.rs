use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::streaming::LayerAddress;

// What a set holds of one page.
pub(crate) enum PageState {
    // Nothing: never read, or its bytes were evicted.
    Unloaded,
    // A worker reads it; setting `cancel` makes the worker skip it if it has not started. The worker reports back
    // either way.
    Reading { cancel: Arc<AtomicBool> },
    // Its bytes are in RAM, in no layer.
    Cached,
    // Its bytes are copying into `layer`.
    Uploading { layer: LayerAddress },
    // In `layer`, read by shaders through the chunk's bindless index `slot`.
    Resident { layer: LayerAddress, slot: u32 },
    // The source has no data for it.
    Absent,
    // Reading it failed; it is not read again.
    Failed,
}

impl PageState {
    // The layer it holds or is copying into.
    pub fn layer(&self) -> Option<LayerAddress> {
        match self {
            PageState::Uploading { layer } | PageState::Resident { layer, .. } => Some(*layer),
            _ => None,
        }
    }
}
