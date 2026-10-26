use std::sync::{Arc, Mutex};

use crate::{AnyEntry, MipDemand, MipEntry, ResourceState, StreamedMips};

/// A reference to a mip-streamed resource, given at once while it loads. It states its holder's demand; the resource
/// keeps resident what all its handles demand together. Cloning it gives a handle with its own copy of the demand.
pub struct MipHandle<K: StreamedMips> {
    pub(crate) entry: Arc<MipEntry<K>>,
    demand: Arc<Mutex<Option<MipDemand>>>,
}

impl<K: StreamedMips> MipHandle<K> {
    pub(crate) fn new(entry: Arc<MipEntry<K>>, demand: Option<MipDemand>) -> MipHandle<K> {
        let demand = Arc::new(Mutex::new(demand));
        entry.acquire();
        entry.add_demand(&demand);

        MipHandle { entry, demand }
    }

    pub fn state(&self) -> ResourceState {
        self.entry.state()
    }

    /// The levels resident now, once the first are ready. The value changes as reloads complete: hold it for a frame,
    /// not longer.
    pub fn get(&self) -> Option<Arc<K>> {
        self.entry.current()
    }

    /// Whether the resident levels reach the finest this handle demands, with no reload toward it under way.
    pub fn is_settled(&self) -> bool {
        let demand = *self.demand.lock().expect("a mip demand's lock is poisoned");

        self.entry.is_settled(demand)
    }

    pub fn set_demand(&self, demand: MipDemand) {
        *self.demand.lock().expect("a mip demand's lock is poisoned") = Some(demand);
    }
}

impl<K: StreamedMips> Clone for MipHandle<K> {
    fn clone(&self) -> MipHandle<K> {
        let demand = *self.demand.lock().expect("a mip demand's lock is poisoned");

        MipHandle::new(Arc::clone(&self.entry), demand)
    }
}

impl<K: StreamedMips> Drop for MipHandle<K> {
    fn drop(&mut self) {
        self.entry.release();
    }
}
