use std::sync::Arc;

use crate::{AnyEntry, Entry, ResourceKind, ResourceState};

/// A reference to a resource, given at once while it loads. Cloning it is cheap; once the last handle (and group
/// holding it) is dropped, the resource follows its retention.
pub struct ResourceHandle<K: ResourceKind> {
    pub(crate) entry: Arc<Entry<K>>,
}

impl<K: ResourceKind> ResourceHandle<K> {
    pub(crate) fn new(entry: Arc<Entry<K>>) -> ResourceHandle<K> {
        entry.acquire();
        ResourceHandle { entry }
    }

    pub fn state(&self) -> ResourceState {
        self.entry.state()
    }

    /// The resource, once ready.
    pub fn get(&self) -> Option<&K> {
        self.entry.value()
    }
}

impl<K: ResourceKind> Clone for ResourceHandle<K> {
    fn clone(&self) -> ResourceHandle<K> {
        ResourceHandle::new(Arc::clone(&self.entry))
    }
}

impl<K: ResourceKind> Drop for ResourceHandle<K> {
    fn drop(&mut self) {
        self.entry.release();
    }
}
