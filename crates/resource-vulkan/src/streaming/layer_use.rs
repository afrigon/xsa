use crate::streaming::{LayerAddress, PageId, PageRetention};

// A page holding a layer, as eviction sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LayerUse {
    pub page: PageId,
    pub layer: LayerAddress,
    pub last_use: u64,
    // Its score while it is wanted.
    pub score: Option<f32>,
    pub retention: PageRetention,
}
