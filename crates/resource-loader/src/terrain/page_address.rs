use super::PageKey;
use crate::utils::CubeFace;

/// Where a page is: its face, its level and its key within them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PageAddress {
    pub face: CubeFace,
    pub level: u32,
    pub key: PageKey,
}
