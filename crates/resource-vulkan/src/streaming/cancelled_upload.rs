use crate::streaming::{LayerAddress, PageId};

// An upload whose page stopped being wanted or lost its layer: its ticket is dropped and its layer taken back.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CancelledUpload {
    pub page: PageId,
    pub layer: LayerAddress,
}
