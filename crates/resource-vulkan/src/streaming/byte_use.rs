use crate::streaming::PageId;

// A page's decoded bytes in RAM, as eviction sees them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ByteUse {
    pub page: PageId,
    pub bytes: u64,
    pub last_use: u64,
    pub score: Option<f32>,
}
