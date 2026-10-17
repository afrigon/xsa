use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use memory_vulkan::BindlessTable;

use crate::{
    AnyEntry, EntryId, Events, LoadContext, LoadEnvironment, LoadPriority, LoadRequest, MemoryUsage, PendingResource,
    Registry, ResourceKind, ResourceState, Retention, Unavailable,
};

// One resource of kind `K`: its file, its handle count and what its load produced so far.
pub(crate) struct Entry<K: ResourceKind> {
    id: EntryId,
    path: PathBuf,
    options: K::Options,
    requester: String,
    size: u64,
    handles: AtomicUsize,
    priority: AtomicUsize,
    // Cached when any request asked for it.
    cached: AtomicBool,
    required: AtomicBool,
    forgotten: AtomicBool,
    released_at: AtomicU64,
    value: OnceLock<K>,
    unavailable: OnceLock<Unavailable>,
    pending: Mutex<Option<Box<dyn PendingResource<K>>>>,
    events: Arc<Mutex<Events>>,
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
            id,
            path: request.path,
            options,
            requester: request.requester,
            size,
            handles: AtomicUsize::new(0),
            priority: AtomicUsize::new(request.priority.index()),
            cached: AtomicBool::new(request.retention == Retention::Cache),
            required: AtomicBool::new(false),
            forgotten: AtomicBool::new(false),
            released_at: AtomicU64::new(0),
            value: OnceLock::new(),
            unavailable: OnceLock::new(),
            pending: Mutex::new(None),
            events,
        }
    }

    pub fn key(&self) -> (PathBuf, K::Options) {
        (self.path.clone(), self.options.clone())
    }

    pub fn value(&self) -> Option<&K> {
        self.value.get()
    }

    pub fn priority(&self) -> LoadPriority {
        LoadPriority::from_index(self.priority.load(Ordering::Acquire))
    }

    // Another request for the same resource: it keeps the most lasting retention and the highest priority.
    pub fn join(&self, request: &LoadRequest) {
        if request.retention == Retention::Cache {
            self.cached.store(true, Ordering::Release);
        }

        if request.priority > self.priority() {
            self.set_priority(request.priority);
        }
    }

    // A required resource already missing or failed is reported again, now as required.
    pub fn require(&self) {
        let was_required = self.required.swap(true, Ordering::AcqRel);

        if !was_required && self.unavailable.get().is_some() {
            self.events().unavailable.push(self.id);
        }
    }

    pub fn settle_unavailable(&self, unavailable: Unavailable) {
        if self.unavailable.set(unavailable).is_ok() {
            self.events().unavailable.push(self.id);
        }
    }

    // On a worker: reads, decodes and queues the uploads, then leaves the pending resource for the main thread.
    pub fn load(self: &Arc<Self>, environment: &LoadEnvironment, uploading: &Mutex<Vec<Arc<dyn AnyEntry>>>) {
        if self.forgotten.load(Ordering::Acquire) {
            return;
        }

        let priority = self.priority();
        let context = LoadContext {
            memory: &environment.memory,
            transfer: &environment.transfer,
            bindless: &environment.bindless,
            name: &self.requester,
            priority: priority.transfer(),
        };

        match K::load(&context, &self.path, &self.options) {
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

    fn events(&self) -> MutexGuard<'_, Events> {
        self.events.lock().expect("the resource events' lock is poisoned")
    }
}

impl<K: ResourceKind> AnyEntry for Entry<K> {
    fn id(&self) -> EntryId {
        self.id
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn state(&self) -> ResourceState {
        if self.value.get().is_some() {
            return ResourceState::Ready;
        }

        match self.unavailable.get() {
            None => ResourceState::Loading,
            Some(Unavailable::Missing) => ResourceState::Missing,
            Some(Unavailable::Failed(error)) => ResourceState::Failed(Arc::clone(error)),
        }
    }

    fn acquire(&self) {
        self.handles.fetch_add(1, Ordering::AcqRel);
    }

    fn release(&self) {
        if self.handles.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.events().released.push(self.id);
        }
    }

    fn handles(&self) -> usize {
        self.handles.load(Ordering::Acquire)
    }

    fn is_cached(&self) -> bool {
        self.cached.load(Ordering::Acquire)
    }

    fn is_required(&self) -> bool {
        self.required.load(Ordering::Acquire)
    }

    fn released_at(&self) -> u64 {
        self.released_at.load(Ordering::Acquire)
    }

    fn set_released_at(&self, sequence: u64) {
        self.released_at.store(sequence, Ordering::Release);
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
        self.priority.store(priority.index(), Ordering::Release);

        if let Some(resource) = self.pending().as_ref() {
            resource.set_priority(priority.transfer());
        }
    }

    fn forget(&self, registry: &mut Registry) {
        self.forgotten.store(true, Ordering::Release);
        registry.kind::<K>().remove(&self.key());
        registry.entries.remove(&self.id);
    }

    fn describe(&self) -> String {
        format!("{} ({})", self.path.display(), self.requester)
    }
}
