use crate::EntryId;
use crate::streaming::{PageFormat, PageOwner};

// What eviction may free: an unreferenced cached resource, a streamed page's cell in its format's pool, or a streamed
// page's decoded bytes in RAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoftOwner {
    Entry(EntryId),
    Cell { pool: PageFormat, page: PageOwner },
    PageBytes(PageOwner),
}
