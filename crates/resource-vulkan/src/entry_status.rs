use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::{EntryId, Events, LoadPriority, LoadRequest, ResourceState, Retention, Unavailable};

// What an entry of any kind tracks besides its value: its file, its handle count, how it was asked for and whether
// it became unavailable.
pub(crate) struct EntryStatus {
    id: EntryId,
    path: PathBuf,
    requester: String,
    size: u64,
    handles: AtomicUsize,
    priority: AtomicUsize,
    // Cached when any request asked for it.
    cached: AtomicBool,
    required: AtomicBool,
    forgotten: AtomicBool,
    released_at: AtomicU64,
    unavailable: OnceLock<Unavailable>,
    events: Arc<Mutex<Events>>,
}

impl EntryStatus {
    pub fn new(id: EntryId, request: LoadRequest, size: u64, events: Arc<Mutex<Events>>) -> EntryStatus {
        EntryStatus {
            id,
            path: request.path,
            requester: request.requester,
            size,
            handles: AtomicUsize::new(0),
            priority: AtomicUsize::new(request.priority.index()),
            cached: AtomicBool::new(request.retention == Retention::Cache),
            required: AtomicBool::new(false),
            forgotten: AtomicBool::new(false),
            released_at: AtomicU64::new(0),
            unavailable: OnceLock::new(),
            events,
        }
    }

    pub fn id(&self) -> EntryId {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn requester(&self) -> &str {
        &self.requester
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    // Ready once the entry holds a value, otherwise loading until it became unavailable.
    pub fn state(&self, ready: bool) -> ResourceState {
        if ready {
            return ResourceState::Ready;
        }

        match self.unavailable.get() {
            None => ResourceState::Loading,
            Some(Unavailable::Missing) => ResourceState::Missing,
            Some(Unavailable::Failed(error)) => ResourceState::Failed(Arc::clone(error)),
        }
    }

    pub fn priority(&self) -> LoadPriority {
        LoadPriority::from_index(self.priority.load(Ordering::Acquire))
    }

    pub fn store_priority(&self, priority: LoadPriority) {
        self.priority.store(priority.index(), Ordering::Release);
    }

    // Another request for the same resource keeps the most lasting retention; true when it raises the priority.
    pub fn join(&self, request: &LoadRequest) -> bool {
        if request.retention == Retention::Cache {
            self.cached.store(true, Ordering::Release);
        }

        request.priority > self.priority()
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

    pub fn acquire(&self) {
        self.handles.fetch_add(1, Ordering::AcqRel);
    }

    pub fn release(&self) {
        if self.handles.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.events().released.push(self.id);
        }
    }

    pub fn handles(&self) -> usize {
        self.handles.load(Ordering::Acquire)
    }

    pub fn is_cached(&self) -> bool {
        self.cached.load(Ordering::Acquire)
    }

    pub fn is_required(&self) -> bool {
        self.required.load(Ordering::Acquire)
    }

    pub fn is_forgotten(&self) -> bool {
        self.forgotten.load(Ordering::Acquire)
    }

    pub fn mark_forgotten(&self) {
        self.forgotten.store(true, Ordering::Release);
    }

    pub fn released_at(&self) -> u64 {
        self.released_at.load(Ordering::Acquire)
    }

    pub fn set_released_at(&self, sequence: u64) {
        self.released_at.store(sequence, Ordering::Release);
    }

    pub fn describe(&self) -> String {
        format!("{} ({})", self.path.display(), self.requester)
    }

    fn events(&self) -> MutexGuard<'_, Events> {
        self.events.lock().expect("the resource events' lock is poisoned")
    }
}
