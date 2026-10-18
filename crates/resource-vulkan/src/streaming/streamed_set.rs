use std::sync::{Arc, Mutex};

use crate::streaming::{PagePlacement, PageRequest, PageSource, Residency, SetId, SetState};

/// A client's paged data, kept resident by what it requests and what the memory budget allows. Dropping it frees
/// every page it holds.
pub struct StreamedSet<S: PageSource> {
    state: Arc<SetState<S>>,
    closed: Arc<Mutex<Vec<SetId>>>,
}

impl<S: PageSource> StreamedSet<S> {
    pub(crate) fn new(state: Arc<SetState<S>>, closed: Arc<Mutex<Vec<SetId>>>) -> StreamedSet<S> {
        StreamedSet { state, closed }
    }

    /// Hands over every page wanted this frame, replacing the previous frame's. `Resources::update` loads them from
    /// the highest score down, and makes the pages no longer wanted evictable.
    pub fn request(&self, requests: impl IntoIterator<Item = PageRequest<S::Key>>) {
        self.state.table().request(requests);
    }

    /// Keeps `key` wanted above every request and never evicts it, until unpinned.
    pub fn pin(&self, key: S::Key) {
        self.state.table().pin(key);
    }

    pub fn unpin(&self, key: S::Key) {
        self.state.table().unpin(key);
    }

    /// Where shaders read `key` from: the page itself once resident, or its nearest resident coarser page until
    /// then. `None` while nothing covering it is resident.
    pub fn page(&self, key: S::Key) -> Option<PagePlacement<S::Key>> {
        let source = self.state.source();

        self.state.table().placement(key, |key| source.coarser(key))
    }

    pub fn set_residency(&self, residency: Residency) {
        self.state.table().residency = residency;
    }
}

impl<S: PageSource> Drop for StreamedSet<S> {
    fn drop(&mut self) {
        self.closed
            .lock()
            .expect("the closed sets' lock is poisoned")
            .push(self.state.id);
    }
}
