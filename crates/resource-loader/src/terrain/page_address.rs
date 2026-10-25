use super::PageKey;
use crate::utils::CubeFace;

/// Where a page is: its face, its level and its key within them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PageAddress {
    pub face: CubeFace,
    pub level: u32,
    pub key: PageKey,
}

impl PageAddress {
    /// The page one level coarser that covers this one, or `None` at level 0.
    pub fn coarser(self) -> Option<PageAddress> {
        Some(PageAddress {
            face: self.face,
            level: self.level.checked_sub(1)?,
            key: self.key.parent(),
        })
    }
}
