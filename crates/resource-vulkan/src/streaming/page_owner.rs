use crate::streaming::{PageId, SetId};

// A page of any set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PageOwner {
    pub set: SetId,
    pub page: PageId,
}
