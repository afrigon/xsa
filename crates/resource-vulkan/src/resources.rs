use std::collections::HashSet;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::available_parallelism;

use anyhow::bail;
use memory_vulkan::{BindlessTable, MemoryContext, TransferQueue};

use crate::streaming::{PageFormat, PageSource, StreamedSet, Streamer};
use crate::{
    Admission, AnyEntry, AvailableMemory, BudgetReading, Entry, EntryId, Events, EvictionPlan, Job, JobOwner,
    LoadEnvironment, LoadPriority, LoadRequest, MemoryBudget, Registry, ResourceGroup, ResourceHandle, ResourceKey,
    ResourceKind, ResourceLimits, ResourceState, SoftUsage, Unavailable, WorkerPool,
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
        let idle_entries = registry.unreferenced_usage(now, &shared.environment.transfer);
        drop(registry);
        self.finish_uploads();
        let room = shared.budget().room(&self.budget_reading());
        let mut admission = Admission::new(room, idle_entries);
        let max_reads = Resources::worker_count(shared.budget().limits) * READS_PER_WORKER;
        let released = shared
            .streamer()
            .update(now, &shared.environment, &shared.pool, max_reads, &mut admission);
        self.finish_evicting(released);
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
                        run: Box::new(move |taken| entry.load(taken, &environment, &uploading)),
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

    // Grows the pools admitted to, then evicts what is least worth keeping until usage is within budget.
    fn keep_within_budget(&self, now: u64) {
        let shared = &self.shared;
        let environment = &shared.environment;
        let mut streamer = shared.streamer();
        streamer.grow(environment);
        drop(streamer);
        let overage = shared.budget().overage(&self.budget_reading());

        if overage.video == 0 && overage.ram == 0 {
            return;
        }

        let mut registry = shared.registry();
        let mut streamer = shared.streamer();
        let mut soft = registry.unreferenced_usage(now, &environment.transfer);
        soft.extend(streamer.soft_usage(now, &environment.transfer));
        soft.sort_by(SoftUsage::eviction_order);
        let plan = EvictionPlan::new(overage, &soft, &streamer.pool_capacities());

        for id in plan.entries {
            let Some(entry) = registry.entries.get(&id).cloned() else {
                continue;
            };
            let video = entry.memory().video;
            registry.forget(entry.as_ref());
            drop(entry);
            self.start_evicting(video);
            self.finish_evicting(video);
        }

        for page in plan.page_bytes {
            streamer.evict_page_bytes(page);
        }

        for shrink in &plan.shrinks {
            let releasing = streamer.shrink(shrink, &environment.memory);
            self.start_evicting(releasing);
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

    // Counts `video` bytes as freed from now, until `finish_evicting` for them.
    fn start_evicting(&self, video: u64) {
        self.shared.evicting_video.fetch_add(video, Ordering::AcqRel);
    }

    // The objects holding `video` bytes evicting were dropped: they stop counting as freed once the deletion queue
    // frees them, after everything queued before.
    fn finish_evicting(&self, video: u64) {
        if video == 0 {
            return;
        }

        let evicting_video = Arc::clone(&self.shared.evicting_video);
        self.shared.environment.memory.defer_action(move |_| {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    use ash::vk;
    use memory_vulkan::{Buffer, MemoryLocation, TestDevice, TexelBlock, TransferPriority};

    use super::*;
    use crate::streaming::PagePool;
    use crate::{LoadContext, MemoryUsage, PageRequest, PendingResource, Residency};

    const MIB: u64 = 1024 * 1024;

    const PAGE_EXTENT: u32 = 64;
    const TEXEL_SIZE: u32 = 4;
    const MAX_FRAMES: usize = 2000;
    // An update that takes longer waited on something.
    const STALL: Duration = Duration::from_millis(250);

    // GPU tests run one at a time: the driver reports video memory per process.
    static GPU: Mutex<()> = Mutex::new(());

    // A quadtree page: its four children at the next level are `index * 4` to `index * 4 + 3`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct QuadKey {
        level: u8,
        index: u32,
    }

    impl QuadKey {
        fn new(level: u8, index: u32) -> QuadKey {
            QuadKey { level, index }
        }

        // Every texel of a page holds this value, so a page read back names itself.
        fn texel(self) -> u32 {
            (self.level as u32) << 24 | self.index
        }
    }

    // Procedural pages. Levels finer than `finest_level` have no data.
    struct SyntheticPages {
        finest_level: u8,
        reads: Arc<AtomicUsize>,
    }

    impl PageSource for SyntheticPages {
        type Key = QuadKey;

        fn coarser(&self, key: QuadKey) -> Option<QuadKey> {
            (key.level > 0).then(|| QuadKey::new(key.level - 1, key.index / 4))
        }

        fn read(&self, key: QuadKey) -> anyhow::Result<Option<Vec<u8>>> {
            self.reads.fetch_add(1, Ordering::AcqRel);

            if key.level > self.finest_level {
                return Ok(None);
            }

            let texels = (PAGE_EXTENT * PAGE_EXTENT) as usize;

            Ok(Some(key.texel().to_le_bytes().repeat(texels)))
        }
    }

    // A resource of `size_mib` MiB of video memory in shared blocks, as costly to load as `load_millis` says.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct BlockOptions {
        size_mib: u64,
        load_millis: u64,
        name: u32,
    }

    struct Block {
        buffer: Buffer,
    }

    struct PendingBlock {
        block: Option<Block>,
    }

    impl ResourceKind for Block {
        type Options = BlockOptions;

        fn load(
            context: &LoadContext,
            _: &std::path::Path,
            options: &BlockOptions,
        ) -> anyhow::Result<Box<dyn PendingResource<Block>>> {
            std::thread::sleep(Duration::from_millis(options.load_millis));
            let buffer = Buffer::new(
                context.memory,
                context.name,
                options.size_mib * MIB,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                MemoryLocation::GpuOnly,
                &context.transfer.sharing(),
            )?;

            Ok(Box::new(PendingBlock {
                block: Some(Block { buffer }),
            }))
        }

        fn memory(&self) -> MemoryUsage {
            MemoryUsage {
                video: self.buffer.size(),
                ram: 0,
            }
        }
    }

    impl PendingResource<Block> for PendingBlock {
        fn poll(&mut self, _: &BindlessTable) -> anyhow::Result<Option<Block>> {
            Ok(self.block.take())
        }

        fn set_priority(&self, _: TransferPriority) {}
    }

    fn block(name: u32, size_mib: u64, load_millis: u64) -> BlockOptions {
        BlockOptions {
            size_mib,
            load_millis,
            name,
        }
    }

    struct Harness {
        // Declaration order is destruction order: everything holding the device before it.
        set: StreamedSet<SyntheticPages>,
        resources: Resources,
        device: TestDevice,
        reads: Arc<AtomicUsize>,
        slowest_update: Duration,
        _gpu: MutexGuard<'static, ()>,
    }

    impl Harness {
        fn new(limits: ResourceLimits, finest_level: u8) -> Harness {
            let gpu = GPU.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let device = TestDevice::new().unwrap();
            let resources = Resources::new(device.memory(), device.transfer(), device.bindless());
            resources.set_limits(limits);
            let reads = Arc::new(AtomicUsize::new(0));
            let source = SyntheticPages {
                finest_level,
                reads: Arc::clone(&reads),
            };
            let set = resources.stream(source, Harness::format(), "synthetic");

            Harness {
                set,
                resources,
                device,
                reads,
                slowest_update: Duration::ZERO,
                _gpu: gpu,
            }
        }

        fn format() -> PageFormat {
            PageFormat {
                format: vk::Format::R32_UINT,
                extent: vk::Extent2D {
                    width: PAGE_EXTENT,
                    height: PAGE_EXTENT,
                },
                texel_block: TexelBlock {
                    extent: vk::Extent2D { width: 1, height: 1 },
                    size: TEXEL_SIZE,
                },
            }
        }

        fn cells_per_chunk() -> usize {
            PagePool::new(Harness::format()).grid().cells() as usize
        }

        fn chunk_size() -> u64 {
            (Harness::cells_per_chunk() * Harness::format().page_size()) as u64
        }

        fn frame(&mut self) {
            let started = Instant::now();
            self.resources.update().unwrap();
            self.slowest_update = self.slowest_update.max(started.elapsed());
            self.device.frame().unwrap();
        }

        // Requests `pages` every frame until `done` holds.
        fn run_until(&mut self, pages: &[PageRequest<QuadKey>], done: impl Fn(&Harness) -> bool) {
            for _ in 0..MAX_FRAMES {
                self.set.request(pages.iter().copied());
                self.frame();

                if done(self) {
                    return;
                }

                std::thread::sleep(Duration::from_millis(1));
            }

            panic!("streaming did not settle in {MAX_FRAMES} frames");
        }

        fn run_frames(&mut self, pages: &[PageRequest<QuadKey>], frames: usize) {
            for _ in 0..frames {
                self.set.request(pages.iter().copied());
                self.frame();
            }
        }

        fn is_resident(&self, key: QuadKey) -> bool {
            self.set.page(key).is_some_and(|placement| placement.page == key)
        }

        fn all_resident(&self, pages: &[PageRequest<QuadKey>]) -> bool {
            pages.iter().all(|request| self.is_resident(request.key))
        }

        // Video memory objects occupy, as the budget counts it.
        fn video_usage(&self) -> u64 {
            let video = self.device.memory().video_memory();

            video.usage - video.unallocated
        }

        fn chunk_count(&self) -> usize {
            self.resources.shared.streamer().chunk_count()
        }

        // Streams `pages` without a limit first, so the transfer queue's staging reaches its high-water mark, then
        // limits video memory to what everything but the pool uses plus `chunks` chunks. Returns the limit.
        fn limit_after_warming_up(&mut self, pages: &[PageRequest<QuadKey>], chunks: f64) -> u64 {
            self.run_until(pages, |harness| harness.all_resident(pages));
            self.run_frames(pages, 10);
            let pool = self.resources.shared.streamer().video_bytes();
            let limit = self.video_usage() - pool + (Harness::chunk_size() as f64 * chunks) as u64;
            self.resources.set_limits(ResourceLimits {
                max_video_memory_usage: Some(limit),
                ..ResourceLimits::default()
            });

            limit
        }

        // Loads every block, waits until all are ready, and returns their handles.
        fn load_blocks(&mut self, blocks: &[BlockOptions]) -> Vec<ResourceHandle<Block>> {
            let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
            let handles: Vec<_> = blocks
                .iter()
                .map(|&options| {
                    self.resources
                        .load::<Block>(LoadRequest::new(manifest, format!("block {}", options.name)), options)
                })
                .collect();
            self.run_until(&[], |_| handles.iter().all(|handle| handle.get().is_some()));

            handles
        }

        fn is_cached(&self, options: BlockOptions) -> bool {
            let key = ResourceKey::<Block> {
                path: concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").into(),
                options,
            };

            self.resources.shared.registry().kind::<Block>().contains_key(&key)
        }

        // Lowers the video memory limit to `below` bytes under what objects occupy now.
        fn limit_below_usage(&self, below: u64) {
            self.resources.set_limits(ResourceLimits {
                max_video_memory_usage: Some(self.video_usage() - below),
                ..ResourceLimits::default()
            });
        }

        fn resident_pages(&self, pages: &[PageRequest<QuadKey>]) -> Vec<PageRequest<QuadKey>> {
            pages
                .iter()
                .copied()
                .filter(|request| self.is_resident(request.key))
                .collect()
        }

        fn assert_holds_its_own_content(&self, key: QuadKey) {
            let placement = self.set.page(key).unwrap();
            let image = self.resources.shared.streamer().chunk_image(placement.slot).unwrap();
            let bytes = self
                .device
                .read_region(
                    &image,
                    vk::ImageLayout::GENERAL,
                    placement.offset,
                    Harness::format().extent,
                    Harness::format().page_size(),
                )
                .unwrap();
            let texels = (PAGE_EXTENT * PAGE_EXTENT) as usize;

            assert_eq!(bytes, key.texel().to_le_bytes().repeat(texels), "{key:?}");
        }

        fn assert_clean(&self) {
            assert_eq!(self.device.validation_messages(), Vec::<String>::new());
            assert!(self.slowest_update < STALL, "an update took {:?}", self.slowest_update);
        }
    }

    // Every page of `level`, scored by `score`.
    fn level(level: u8, score: impl Fn(u32) -> f32) -> Vec<PageRequest<QuadKey>> {
        (0..4u32.pow(level as u32))
            .map(|index| PageRequest {
                key: QuadKey::new(level, index),
                score: score(index),
            })
            .collect()
    }

    fn levels(levels: std::ops::RangeInclusive<u8>) -> Vec<PageRequest<QuadKey>> {
        levels
            .flat_map(|page_level| level(page_level, move |_| 100.0 - page_level as f32))
            .collect()
    }

    #[test]
    fn wanted_pages_become_resident_and_hold_their_own_content() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let pages = levels(0..=3);
        harness.run_until(&pages, |harness| harness.all_resident(&pages));
        let texels = (PAGE_EXTENT * PAGE_EXTENT) as usize;

        for request in &pages {
            let placement = harness.set.page(request.key).unwrap();
            let image = harness.resources.shared.streamer().chunk_image(placement.slot).unwrap();
            let bytes = harness
                .device
                .read_region(
                    &image,
                    vk::ImageLayout::GENERAL,
                    placement.offset,
                    Harness::format().extent,
                    Harness::format().page_size(),
                )
                .unwrap();

            assert_eq!(
                bytes,
                request.key.texel().to_le_bytes().repeat(texels),
                "{:?}",
                request.key
            );
        }

        harness.assert_clean();
    }

    #[test]
    fn a_page_without_data_falls_back_to_its_nearest_resident_coarser_page() {
        let mut harness = Harness::new(ResourceLimits::default(), 1);
        let pages = levels(0..=3);
        harness.run_until(&pages, |harness| harness.all_resident(&levels(0..=1)));
        harness.run_frames(&pages, 10);

        assert_eq!(
            harness.set.page(QuadKey::new(3, 37)).map(|placement| placement.page),
            Some(QuadKey::new(1, 2))
        );
        harness.assert_clean();
    }

    #[test]
    fn under_a_small_budget_pages_load_in_score_order_without_thrashing() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let pages = level(6, |index| 4096.0 - index as f32);
        let budget = harness.limit_after_warming_up(&pages, 1.5);
        let most_important = &pages[..Harness::cells_per_chunk()];
        harness.run_until(&pages, |harness| harness.all_resident(most_important));
        let reads = harness.reads.load(Ordering::Acquire);
        harness.run_frames(&pages, 60);

        assert_eq!(harness.chunk_count(), 1);
        assert!(harness.all_resident(most_important));
        assert!(!harness.is_resident(pages[Harness::cells_per_chunk()].key));
        assert_eq!(
            harness.reads.load(Ordering::Acquire),
            reads,
            "settled pages are read again"
        );
        assert!(harness.video_usage() <= budget + Harness::chunk_size());
        harness.assert_clean();
    }

    #[test]
    fn the_pool_grows_and_shrinks_with_the_budget() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let pages: Vec<_> = levels(4..=5)
            .into_iter()
            .chain(level(6, |index| 50.0 - index as f32 / 4096.0))
            .collect();
        harness.run_until(&pages, |harness| harness.all_resident(&pages));
        let grown = harness.chunk_count();

        assert_eq!(grown, pages.len().div_ceil(Harness::cells_per_chunk()));

        let budget = harness.video_usage() - 2 * Harness::chunk_size();
        harness.resources.set_limits(ResourceLimits {
            max_video_memory_usage: Some(budget),
            ..ResourceLimits::default()
        });
        let most_important = levels(4..=5);
        harness.run_until(&pages, |harness| {
            harness.chunk_count() <= grown - 2 && harness.all_resident(&most_important)
        });

        assert!(harness.video_usage() <= budget + Harness::chunk_size());

        harness.resources.set_limits(ResourceLimits::default());
        harness.run_until(&pages, |harness| harness.all_resident(&pages));

        assert_eq!(harness.chunk_count(), grown);
        harness.assert_clean();
    }

    #[test]
    fn reads_no_longer_wanted_are_cancelled() {
        let mut harness = Harness::new(
            ResourceLimits {
                max_cpu_cores: NonZeroUsize::new(2),
                read_delay: Some(Duration::from_millis(50)),
                ..ResourceLimits::default()
            },
            8,
        );
        let pages = level(5, |_| 1.0);
        harness.run_frames(&pages, 2);
        harness.run_frames(&[], 1);
        std::thread::sleep(Duration::from_millis(300));
        harness.run_frames(&[], 2);

        assert!(harness.reads.load(Ordering::Acquire) <= 2 * READS_PER_WORKER);
        harness.assert_clean();
    }

    #[test]
    fn pinned_pages_survive_budget_pressure() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let pages = level(6, |_| 1000.0);
        harness.limit_after_warming_up(&pages, 1.5);
        let pinned = levels(0..=1);

        for request in &pinned {
            harness.set.pin(request.key);
        }

        harness.run_until(&pages, |harness| harness.all_resident(&pinned));
        harness.run_frames(&pages, 60);

        assert!(harness.all_resident(&pinned));
        harness.assert_clean();
    }

    #[test]
    fn a_frozen_set_keeps_its_pages() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let held = levels(0..=2);
        harness.run_until(&held, |harness| harness.all_resident(&held));
        harness.set.set_residency(Residency::Frozen);
        let reads = harness.reads.load(Ordering::Acquire);
        harness.run_frames(&levels(3..=3), 30);

        assert!(harness.all_resident(&held));
        assert_eq!(harness.reads.load(Ordering::Acquire), reads);

        harness.set.set_residency(Residency::Live);
        harness.run_until(&levels(3..=3), |harness| harness.all_resident(&levels(3..=3)));
        harness.assert_clean();
    }

    #[test]
    fn of_cached_resources_the_cheapest_to_reload_goes_first_and_only_what_is_needed() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let cheap = block(1, 16, 0);
        let costly = block(2, 16, 300);
        let other_cheap = block(3, 16, 0);
        drop(harness.load_blocks(&[cheap, costly, other_cheap]));
        harness.run_frames(&[], 2);
        harness.limit_below_usage(8 * MIB);
        harness.run_frames(&[], 30);

        assert!(harness.is_cached(costly));
        assert_eq!(
            [cheap, other_cheap]
                .into_iter()
                .filter(|&options| harness.is_cached(options))
                .count(),
            1,
            "exactly one cheap block goes"
        );
        harness.assert_clean();
    }

    #[test]
    fn of_cached_resources_as_costly_to_reload_the_larger_goes_first() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let large = block(1, 64, 100);
        let small = block(2, 8, 100);
        drop(harness.load_blocks(&[large, small]));
        harness.run_frames(&[], 2);
        harness.limit_below_usage(4 * MIB);
        harness.run_frames(&[], 30);

        assert!(!harness.is_cached(large));
        assert!(harness.is_cached(small));
        harness.assert_clean();
    }

    #[test]
    fn of_cached_resources_the_one_unused_longest_goes_first() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let stale = block(1, 16, 50);
        let recent = block(2, 16, 50);
        let mut handles = harness.load_blocks(&[stale, recent]);
        drop(handles.remove(0));
        harness.run_frames(&[], 60);
        drop(handles);
        harness.run_frames(&[], 2);
        harness.limit_below_usage(8 * MIB);
        harness.run_frames(&[], 30);

        assert!(!harness.is_cached(stale));
        assert!(harness.is_cached(recent));
        harness.assert_clean();
    }

    #[test]
    fn an_unused_cached_resource_goes_before_wanted_pages() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let cached = block(1, 16, 1000);
        drop(harness.load_blocks(&[cached]));
        let pages = levels(4..=5);
        harness.run_until(&pages, |harness| harness.all_resident(&pages));
        harness.limit_below_usage(8 * MIB);
        harness.run_frames(&pages, 30);

        assert!(!harness.is_cached(cached));
        assert!(harness.all_resident(&pages));
        harness.assert_clean();
    }

    #[test]
    fn a_shrinking_pool_keeps_its_most_important_pages_and_moves_them_intact() {
        let mut harness = Harness::new(ResourceLimits::default(), 8);
        let loading = level(6, |index| 8192.0 - index as f32);
        harness.run_until(&loading, |harness| harness.all_resident(&loading));
        let chunks = harness.chunk_count();

        assert!(chunks >= 2);

        // Scores no longer follow the order pages were placed in, so the least important are spread across chunks.
        let pages = level(6, |index| ((index * 7919) % 4096) as f32 + 1.0);
        let before: HashMap<QuadKey, (u32, vk::Offset2D)> = pages
            .iter()
            .map(|request| {
                let placement = harness.set.page(request.key).unwrap();
                (request.key, (placement.slot, placement.offset))
            })
            .collect();
        harness.run_frames(&pages, 2);
        harness.limit_below_usage(Harness::chunk_size() / 2);
        harness.run_until(&pages, |harness| harness.chunk_count() < chunks);
        harness.run_frames(&pages, 30);
        let resident = harness.resident_pages(&pages);
        let lowest_resident = resident
            .iter()
            .map(|request| request.score)
            .fold(f32::INFINITY, f32::min);
        let highest_dropped = pages
            .iter()
            .filter(|request| !harness.is_resident(request.key))
            .map(|request| request.score)
            .fold(0.0, f32::max);

        assert_eq!(harness.chunk_count(), chunks - 1);
        assert!(
            lowest_resident > highest_dropped,
            "a page of score {highest_dropped} fell back while one of {lowest_resident} stayed"
        );

        let moved: Vec<_> = resident
            .iter()
            .filter(|request| {
                let placement = harness.set.page(request.key).unwrap();
                before[&request.key] != (placement.slot, placement.offset)
            })
            .collect();

        assert!(!moved.is_empty());

        for request in moved {
            harness.assert_holds_its_own_content(request.key);
        }

        harness.assert_clean();
    }

    #[test]
    fn pages_that_do_not_fit_in_ram_are_not_read_again_and_again() {
        let mut harness = Harness::new(
            ResourceLimits {
                max_memory_usage: Some(4 * MIB),
                ..ResourceLimits::default()
            },
            8,
        );
        let pages = level(6, |index| 4096.0 - index as f32);
        harness.run_frames(&pages, 120);
        let reads = harness.reads.load(Ordering::Acquire);
        harness.run_frames(&pages, 60);

        assert_eq!(harness.reads.load(Ordering::Acquire), reads, "pages are read again");
        assert!(harness.resources.shared.streamer().ram_bytes() <= 4 * MIB);
        assert!(harness.is_resident(pages[0].key));
        harness.assert_clean();
    }
}
