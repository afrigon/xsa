use std::collections::HashSet;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::available_parallelism;

use anyhow::bail;
use memory_vulkan::{BindlessTable, MemoryContext, TransferQueue};

use crate::streaming::{PageFormat, PageSource, StreamedSet, Streamer};
use crate::{
    AnyEntry, AvailableMemory, BudgetReading, Entry, EntryId, Events, Job, JobOwner, LoadEnvironment, LoadPriority,
    LoadRequest, MemoryBudget, Registry, ResourceGroup, ResourceHandle, ResourceKey, ResourceKind, ResourceLimits,
    ResourceState, SoftOwner, SoftUsage, Unavailable, WorkerPool,
};

// Page reads queued ahead per worker: enough to keep every worker busy, few enough that the order follows each frame's
// scores.
const READS_PER_WORKER: usize = 2;

/// Loads resource files in the background and frees them once nothing uses them.
///
/// Cloning it is cheap, and clones share the same resources across threads. Loads start at once on a worker pool;
/// `update`, once per frame on the thread that draws, finishes them and keeps memory within budget.
#[derive(Clone)]
pub struct Resources {
    shared: Arc<SharedResources>,
}

struct SharedResources {
    // First, so the workers stop before what they load into drops.
    pool: WorkerPool,
    environment: Arc<LoadEnvironment>,
    registry: Mutex<Registry>,
    events: Arc<Mutex<Events>>,
    // Entries whose uploads are queued, polled by `update`.
    uploading: Arc<Mutex<Vec<Arc<dyn AnyEntry>>>>,
    budget: Mutex<MemoryBudget>,
    available_memory: Mutex<Option<AvailableMemory>>,
    // Evicted video memory the deletion queue has not freed yet.
    evicting_video: Arc<AtomicU64>,
    streamer: Mutex<Streamer>,
    // Counts updates: the clock last uses are measured in.
    now: AtomicU64,
}

impl Resources {
    pub fn new(memory: &MemoryContext, transfer: &TransferQueue, bindless: &BindlessTable) -> Resources {
        let available_memory = AvailableMemory::new()
            .inspect_err(|err| tracing::warn!("resources keep within video memory only: {err:#}"))
            .ok();

        Resources {
            shared: Arc::new(SharedResources {
                pool: WorkerPool::new(Resources::worker_count(ResourceLimits::default())),
                environment: Arc::new(LoadEnvironment {
                    memory: memory.clone(),
                    transfer: transfer.clone(),
                    bindless: bindless.clone(),
                }),
                registry: Mutex::new(Registry::default()),
                events: Arc::new(Mutex::new(Events::default())),
                uploading: Arc::new(Mutex::new(Vec::new())),
                budget: Mutex::new(MemoryBudget::default()),
                available_memory: Mutex::new(available_memory),
                evicting_video: Arc::new(AtomicU64::new(0)),
                streamer: Mutex::new(Streamer::new()),
                now: AtomicU64::new(0),
            }),
        }
    }

    /// Starts loading an optional resource, or shares the one already loaded from the same file with the same
    /// options. A missing or failed one logs a warning; the caller picks its fallback.
    pub fn load<K: ResourceKind>(&self, request: LoadRequest, options: K::Options) -> ResourceHandle<K> {
        self.request(request, options)
    }

    /// Starts loading a resource the game cannot run without: if it is missing or fails, `update` returns the error.
    pub fn load_required<K: ResourceKind>(&self, request: LoadRequest, options: K::Options) -> ResourceHandle<K> {
        let handle = self.request(request, options);
        handle.entry.require();

        handle
    }

    /// Registers paged data that streams through `source` in pages of `format`, named `name` in messages. The set's
    /// pages load and evict in `update`, by what it requests and what the memory budget allows.
    pub fn stream<S: PageSource>(&self, source: S, format: PageFormat, name: impl Into<String>) -> StreamedSet<S> {
        self.shared.streamer().register(source, format, name.into())
    }

    /// Moves every resource of `group` still loading to `priority`, including uploads already queued.
    pub fn set_priority(&self, group: &ResourceGroup, priority: LoadPriority) {
        for entry in group.entries() {
            entry.set_priority(priority);
            self.shared.pool.reprioritize(entry.id(), priority);
        }
    }

    pub fn set_limits(&self, limits: ResourceLimits) {
        self.shared.pool.set_worker_count(Resources::worker_count(limits));
        self.shared.pool.set_read_delay(limits.read_delay);
        self.shared.budget().limits = limits;
    }

    /// Finishes the loads whose uploads completed, reports missing and failed files, frees what is no longer used,
    /// streams the pages streamed sets want and keeps memory within budget. Fails when a required resource is missing
    /// or failed.
    pub fn update(&self) -> anyhow::Result<()> {
        let shared = &self.shared;
        let now = shared.now.fetch_add(1, Ordering::AcqRel) + 1;
        let events = std::mem::take(&mut *shared.events());
        let mut registry = shared.registry();
        let required_failures = Resources::report(&registry, &events.unavailable);

        for id in events.released {
            let Some(entry) = registry.entries.get(&id).cloned() else {
                continue;
            };

            if entry.handles() > 0 {
                continue;
            }

            let has_value = !matches!(entry.state(), ResourceState::Missing | ResourceState::Failed(_));

            if entry.is_cached() && has_value {
                registry.keep_unreferenced(entry.as_ref(), now);
            } else {
                registry.forget(entry.as_ref());
            }
        }

        registry.compact();
        drop(registry);
        self.finish_uploads();
        let max_reads = Resources::worker_count(shared.budget().limits) * READS_PER_WORKER;
        shared
            .streamer()
            .update(now, &shared.environment, &shared.pool, max_reads);
        self.keep_within_budget(now);

        if !required_failures.is_empty() {
            bail!("required resources are unavailable:\n{}", required_failures.join("\n"));
        }

        Ok(())
    }

    fn request<K: ResourceKind>(&self, request: LoadRequest, options: K::Options) -> ResourceHandle<K> {
        let shared = &self.shared;
        let mut registry = shared.registry();
        let key = ResourceKey::<K> {
            path: request.path.clone(),
            options,
        };

        if let Some(entry) = registry.kind::<K>().get(&key).cloned() {
            entry.join(&request);
            shared.pool.reprioritize(entry.id(), entry.priority());

            return ResourceHandle::new(entry);
        }

        let metadata = std::fs::metadata(&request.path);
        let size = metadata.as_ref().map_or(0, |metadata| metadata.len());
        let priority = request.priority;
        let id = registry.next_id();
        let entry = Arc::new(Entry::<K>::new(
            id,
            request,
            key.options,
            size,
            Arc::clone(&shared.events),
        ));
        registry.insert(Arc::clone(&entry));
        let handle = ResourceHandle::new(Arc::clone(&entry));
        drop(registry);

        match metadata {
            Err(err) if err.kind() == ErrorKind::NotFound => entry.settle_unavailable(Unavailable::Missing),
            Err(err) => entry.settle_unavailable(Unavailable::Failed(Arc::new(err.into()))),
            Ok(_) => {
                let environment = Arc::clone(&shared.environment);
                let uploading = Arc::clone(&shared.uploading);
                shared.pool.push(
                    priority,
                    Job {
                        owner: JobOwner::Entry(id),
                        run: Box::new(move || entry.load(&environment, &uploading)),
                    },
                );
            }
        }

        handle
    }

    // Logs each optional resource that became unavailable, and returns the required ones' messages.
    fn report(registry: &Registry, unavailable: &[EntryId]) -> Vec<String> {
        let mut required_failures = Vec::new();
        let unique: HashSet<_> = unavailable.iter().collect();

        for entry in unique.into_iter().filter_map(|id| registry.entries.get(id)) {
            let reason = match entry.state() {
                ResourceState::Missing => "the file does not exist".to_owned(),
                ResourceState::Failed(error) => format!("{error:#}"),
                ResourceState::Loading | ResourceState::Ready => continue,
            };
            let message = format!("{}: {reason}", entry.describe());

            if entry.is_required() {
                required_failures.push(message);
            } else {
                tracing::warn!("{message}");
            }
        }

        required_failures
    }

    fn finish_uploads(&self) {
        let bindless = &self.shared.environment.bindless;
        let mut uploading = std::mem::take(&mut *self.shared.uploading());
        uploading.retain(|entry| !entry.poll(bindless));
        self.shared.uploading().extend(uploading);
    }

    fn keep_within_budget(&self, now: u64) {
        let shared = &self.shared;
        let mut reading = self.budget_reading();
        let budget = shared.budget();
        let mut streamer = shared.streamer();

        if streamer.is_starved() {
            let registry = shared.registry();
            let unreferenced_video: u64 = registry.unreferenced_usage().map(|usage| usage.memory.video).sum();
            drop(registry);
            let idle_chunks: u64 = streamer
                .chunk_usage()
                .iter()
                .filter(|usage| usage.last_use < now)
                .map(|usage| usage.memory.video)
                .sum();
            let room = budget.video_room(&reading, unreferenced_video + idle_chunks);

            if streamer.grow(room, now, &shared.environment) {
                reading.video = shared.environment.memory.video_memory();
            }
        }

        let overage = budget.overage(&reading);
        drop(budget);

        if overage.video == 0 && overage.ram == 0 {
            return;
        }

        let mut registry = shared.registry();
        let mut soft: Vec<SoftUsage> = registry.unreferenced_usage().collect();

        if overage.video > 0 {
            soft.extend(streamer.chunk_usage());
        }

        if overage.ram > 0 {
            soft.extend(streamer.page_bytes_usage());
        }

        soft.sort_by(SoftUsage::eviction_order);
        let evictions = MemoryBudget::evictions(overage, soft.into_iter().map(|usage| (usage.owner, usage.memory)));

        for owner in evictions {
            match owner {
                SoftOwner::Entry(id) => {
                    let Some(entry) = registry.entries.get(&id).cloned() else {
                        continue;
                    };
                    let video = entry.memory().video;
                    registry.forget(entry.as_ref());
                    drop(entry);
                    self.count_evicting(video);
                }
                SoftOwner::Chunk(chunk) => {
                    if let Some(chunk) = streamer.release_chunk(chunk) {
                        self.count_evicting(chunk.size);
                    }
                }
                SoftOwner::PageBytes(page) => streamer.evict_page_bytes(page),
            }
        }
    }

    fn budget_reading(&self) -> BudgetReading {
        let shared = &self.shared;
        let available_ram = shared
            .available_memory
            .lock()
            .expect("the available memory's lock is poisoned")
            .as_mut()
            .and_then(|memory| {
                memory
                    .bytes()
                    .inspect_err(|err| tracing::warn!("reading the available memory: {err:#}"))
                    .ok()
            });
        let held_ram = shared
            .registry()
            .entries
            .values()
            .map(|entry| entry.memory().ram)
            .sum::<u64>()
            + shared.streamer().ram_bytes();

        BudgetReading {
            video: shared.environment.memory.video_memory(),
            evicting_video: shared.evicting_video.load(Ordering::Acquire),
            held_ram,
            available_ram,
        }
    }

    // Counts `video` bytes as freed until the deletion queue frees them.
    fn count_evicting(&self, video: u64) {
        let shared = &self.shared;
        shared.evicting_video.fetch_add(video, Ordering::AcqRel);
        let evicting_video = Arc::clone(&shared.evicting_video);
        shared.environment.memory.defer_action(move |_| {
            evicting_video.fetch_sub(video, Ordering::AcqRel);
        });
    }

    fn worker_count(limits: ResourceLimits) -> usize {
        let cores = available_parallelism().map_or(1, |cores| cores.get());

        limits.max_cpu_cores.map_or(cores, |limit| cores.min(limit.get()))
    }
}

impl SharedResources {
    fn registry(&self) -> MutexGuard<'_, Registry> {
        self.registry.lock().expect("the resource registry's lock is poisoned")
    }

    fn events(&self) -> MutexGuard<'_, Events> {
        self.events.lock().expect("the resource events' lock is poisoned")
    }

    fn uploading(&self) -> MutexGuard<'_, Vec<Arc<dyn AnyEntry>>> {
        self.uploading
            .lock()
            .expect("the uploading resources' lock is poisoned")
    }

    fn budget(&self) -> MutexGuard<'_, MemoryBudget> {
        self.budget.lock().expect("the memory budget's lock is poisoned")
    }

    fn streamer(&self) -> MutexGuard<'_, Streamer> {
        self.streamer.lock().expect("the streamer's lock is poisoned")
    }
}
