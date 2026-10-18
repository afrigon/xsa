use std::sync::Arc;

use crate::streaming::PageState;

// One page a set knows of.
pub(crate) struct PageRecord<K> {
    pub key: K,
    pub state: PageState,
    // The decoded page, kept in RAM while the budget allows, whether or not a layer holds it.
    pub bytes: Option<Arc<[u8]>>,
    // The update it was last wanted or pinned in.
    pub last_use: u64,
    // Its score while it is wanted.
    pub score: Option<f32>,
}
