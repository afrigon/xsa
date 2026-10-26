use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use memory_vulkan::BindlessTable;

use crate::{
    AnyEntry, EntryId, EntryStatus, Events, LoadContext, LoadEnvironment, LoadPriority, LoadRequest, MemoryUsage,
    PendingResource, Registry, ResourceKey, ResourceKind, ResourceState, Unavailable,
};

// One resource of kind `K`: its status and what its load produced so far.
pub(crate) struct Entry<K: ResourceKind> {
    status: EntryStatus,
    options: K::Options,
    value: OnceLock<K>,
    pending: Mutex<Option<Box<dyn PendingResource<K>>>>,
}

impl<K: ResourceKind> Entry<K> {
    pub fn new(
        id: EntryId,
        request: LoadRequest,
        options: K::Options,
        size: u64,
        events: Arc<Mutex<Events>>,
    ) -> Entry<K> {
        Entry {
            status: EntryStatus::new(id, request, size, events),
            options,
            value: OnceLock::new(),
            pending: Mutex::new(None),
        }
    }

    pub fn key(&self) -> ResourceKey<K> {
        ResourceKey {
            path: self.status.path().to_path_buf(),
            options: self.options.clone(),
        }
    }

    pub fn value(&self) -> Option<&K> {
        self.value.get()
    }

    pub fn priority(&self) -> LoadPriority {
        self.status.priority()
    }

    // Another request for the same resource: it keeps the most lasting retention and the highest priority.
    pub fn join(&self, request: &LoadRequest) {
        if self.status.join(request) {
            self.set_priority(request.priority);
        }
    }

    pub fn require(&self) {
        self.status.require();
    }

    pub fn settle_unavailable(&self, unavailable: Unavailable) {
        self.status.settle_unavailable(unavailable);
    }

    // On a worker: reads, decodes and queues the uploads, then leaves the pending resource for the main thread.
    pub fn load(self: &Arc<Self>, environment: &LoadEnvironment, uploading: &Mutex<Vec<Arc<dyn AnyEntry>>>) {
        if self.status.is_forgotten() {
            return;
        }

        let priority = self.priority();
        let context = LoadContext {
            memory: &environment.memory,
            transfer: &environment.transfer,
            bindless: &environment.bindless,
            name: self.status.requester(),
            priority: priority.transfer(),
        };

        match K::load(&context, self.status.path(), &self.options) {
            Ok(resource) => {
                let mut pending = self.pending();
                let current = self.priority();

                // `set_priority` stores the priority before it looks for a pending resource, so a change made while
                // this one loaded is seen here.
                if current != priority {
                    resource.set_priority(current.transfer());
                }

                *pending = Some(resource);
                drop(pending);
                uploading
                    .lock()
                    .expect("the uploading resources' lock is poisoned")
                    .push(Arc::clone(self) as Arc<dyn AnyEntry>);
            }
            Err(err) => self.settle_unavailable(Unavailable::Failed(Arc::new(err))),
        }
    }

    fn pending(&self) -> MutexGuard<'_, Option<Box<dyn PendingResource<K>>>> {
        self.pending.lock().expect("a pending resource's lock is poisoned")
    }
}

impl<K: ResourceKind> AnyEntry for Entry<K> {
    fn id(&self) -> EntryId {
        self.status.id()
    }

    fn size(&self) -> u64 {
        self.status.size()
    }

    fn state(&self) -> ResourceState {
        self.status.state(self.value.get().is_some())
    }

    fn acquire(&self) {
        self.status.acquire();
    }

    fn release(&self) {
        self.status.release();
    }

    fn handles(&self) -> usize {
        self.status.handles()
    }

    fn is_cached(&self) -> bool {
        self.status.is_cached()
    }

    fn is_required(&self) -> bool {
        self.status.is_required()
    }

    fn released_at(&self) -> u64 {
        self.status.released_at()
    }

    fn set_released_at(&self, sequence: u64) {
        self.status.set_released_at(sequence);
    }

    fn memory(&self) -> MemoryUsage {
        self.value.get().map_or(MemoryUsage::default(), K::memory)
    }

    fn poll(&self, bindless: &BindlessTable) -> bool {
        let mut pending = self.pending();
        let Some(resource) = pending.as_mut() else {
            return true;
        };

        match resource.poll(bindless) {
            Ok(None) => false,
            Ok(Some(value)) => {
                *pending = None;
                let _ = self.value.set(value);
                true
            }
            Err(err) => {
                *pending = None;
                drop(pending);
                self.settle_unavailable(Unavailable::Failed(Arc::new(err)));
                true
            }
        }
    }

    fn set_priority(&self, priority: LoadPriority) {
        self.status.store_priority(priority);

        if let Some(resource) = self.pending().as_ref() {
            resource.set_priority(priority.transfer());
        }
    }

    fn forget(&self, registry: &mut Registry) {
        self.status.mark_forgotten();
        registry.kind::<K>().remove(&self.key());
        registry.entries.remove(&self.id());
    }

    fn describe(&self) -> String {
        self.status.describe()
    }
}
