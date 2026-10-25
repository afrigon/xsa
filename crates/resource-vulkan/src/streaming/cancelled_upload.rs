use crate::streaming::{CellAddress, PageId};

// An upload whose page stopped being wanted or lost its cell: its ticket is dropped and its cell taken back.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CancelledUpload {
    pub page: PageId,
    pub cell: CellAddress,
}
