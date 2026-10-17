use std::sync::Arc;

use crate::AnyEntry;

// A handle of any kind, as a group holds it.
pub(crate) struct AnyHandle {
    pub entry: Arc<dyn AnyEntry>,
}

impl AnyHandle {
    pub fn new(entry: Arc<dyn AnyEntry>) -> AnyHandle {
        entry.acquire();
        AnyHandle { entry }
    }
}

impl Drop for AnyHandle {
    fn drop(&mut self) {
        self.entry.release();
    }
}
