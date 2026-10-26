use crate::EntryId;

// What eviction may free: an unreferenced cached resource, or the retained levels of a mip-streamed resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoftOwner {
    Entry(EntryId),
    RetainedLevels(EntryId),
}
