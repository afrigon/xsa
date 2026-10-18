// Whether a page may be evicted. Pinned pages take the lowest free layers, so they gather in few chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PageRetention {
    Evictable,
    Pinned,
}
