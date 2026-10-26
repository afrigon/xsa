use std::collections::HashSet;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::available_parallelism;
use std::time::Instant;

use anyhow::bail;
use ash::vk;
use memory_vulkan::{BindlessTable, MemoryContext, TransferQueue};

use crate::{
    AnyEntry, AvailableMemory, BudgetReading, Entry, EntryId, Events, Job, LoadEnvironment, LoadPriority, LoadRequest,
    MemoryBudget, MipEntry, MipHandle, Registry, ResourceGroup, ResourceHandle, ResourceKey, ResourceKind,
    ResourceLimits, ResourceState, SoftOwner, SoftUsage, StreamedMips, Unavailable, WorkerPool,
};

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

    /// Starts loading a mip-streamed resource, or shares the one already loaded from the same file with the same
    /// options. Its first load reads only the smallest level until a handle states a demand.
    pub fn load_mips<K: StreamedMips>(&self, request: LoadRequest, options: K::Options) -> MipHandle<K> {
        let shared = &self.shared;
        let mut registry = shared.registry();
        let key = ResourceKey::<K::Options> {
            path: request.path.clone(),
            options,
        };

        if let Some(entry) = registry.mip_kind::<K>().get(&key).cloned() {
            entry.join(&request);

            return MipHandle::new(entry, None);
        }

        let metadata = std::fs::metadata(&request.path);
        let size = metadata.as_ref().map_or(0, |metadata| metadata.len());
        let id = registry.next_id();
        let entry = Arc::new(MipEntry::<K>::new(
            id,
            request,
            key.options,
            size,
            Arc::clone(&shared.events),
        ));
        registry.insert_mips(Arc::clone(&entry));
        let handle = MipHandle::new(Arc::clone(&entry), None);
        drop(registry);

        match metadata {
            Err(err) if err.kind() == ErrorKind::NotFound => entry.settle_unavailable(Unavailable::Missing),
            Err(err) => entry.settle_unavailable(Unavailable::Failed(Arc::new(err.into()))),
            Ok(_) => {}
        }

        handle
    }

    /// Records, into a frame's command buffer on the graphics queue, the copy of the levels each finished reload of a
    /// mip-streamed resource shares with its current levels, then makes the reloads current. The copies order after
    /// earlier frames' sampling; a frame that read a handle before this call samples the previous levels, which stay
    /// valid until no recording open now can read them.
    pub fn record_mip_copies(&self, command_buffer: vk::CommandBuffer) {
        let mip_entries: Vec<_> = self.shared.registry().mip_entries.values().cloned().collect();
        let device = self.shared.environment.memory.device();

        for entry in mip_entries {
            entry.record_copies(device, command_buffer);
        }
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
    /// reloads mip-streamed resources toward their demand and keeps memory within budget. Fails when a required
    /// resource is missing or failed.
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
        self.update_mip_residency(now);
        self.keep_within_budget();

        if !required_failures.is_empty() {
            bail!("required resources are unavailable:\n{}", required_failures.join("\n"));
        }

        Ok(())
    }

    fn request<K: ResourceKind>(&self, request: LoadRequest, options: K::Options) -> ResourceHandle<K> {
        let shared = &self.shared;
        let mut registry = shared.registry();
        let key = ResourceKey::<K::Options> {
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
                        entry: id,
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

    fn update_mip_residency(&self, update: u64) {
        let shared = &self.shared;
        let mip_entries: Vec<_> = shared.registry().mip_entries.values().cloned().collect();
        let now = Instant::now();

        for entry in mip_entries {
            entry.update_residency(update, now, &shared.environment, &shared.uploading, &shared.pool);
        }
    }

    fn keep_within_budget(&self) {
        let shared = &self.shared;
        let reading = self.budget_reading();
        let overage = shared.budget().overage(&reading);

        if overage.video == 0 && overage.ram == 0 {
            return;
        }

        let mut registry = shared.registry();
        let mut soft: Vec<SoftUsage> = registry.unreferenced_usage().collect();
        soft.extend(registry.mip_entries.values().filter_map(|entry| entry.retained_usage()));
        soft.sort_by_key(|usage| usage.last_use);
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
                SoftOwner::RetainedLevels(id) => {
                    if let Some(entry) = registry.mip_entries.get(&id) {
                        entry.release_retained();
                    }
                }
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
            .sum::<u64>();

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
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ash::vk;
    use memory_vulkan::TestDevice;

    use super::*;
    use crate::{CubeMap, MipDemand, StreamedMips};

    const TEXEL_SIZE: u32 = 4;
    const MAX_FRAMES: usize = 2000;

    // GPU tests run one at a time: the driver reports video memory per process.
    static GPU: Mutex<()> = Mutex::new(());

    const KTX2_IDENTIFIER: &[u8; 12] = b"\xabKTX 20\xbb\r\n\x1a\n";
    const KTX2_HEADER_SIZE: usize = 80;
    const KTX2_LEVEL_ENTRY_SIZE: usize = 24;
    const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
    const CUBE_FACES: usize = 6;
    const CUBE_SIDE: u32 = 4;
    const CUBE_LEVELS: u32 = 3;

    // An uncompressed RGBA8 cube map whose every texel of a level holds that level's number.
    fn write_tagged_cube(path: &std::path::Path) {
        let levels: Vec<Vec<u8>> = (0..CUBE_LEVELS)
            .map(|level| {
                let side = (CUBE_SIDE >> level) as usize;
                vec![level as u8; side * side * TEXEL_SIZE as usize * CUBE_FACES]
            })
            .collect();
        let mut bytes = KTX2_IDENTIFIER.to_vec();

        for word in [
            VK_FORMAT_R8G8B8A8_UNORM,
            1,
            CUBE_SIDE,
            CUBE_SIDE,
            0,
            0,
            CUBE_FACES as u32,
            CUBE_LEVELS,
            0,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        bytes.resize(KTX2_HEADER_SIZE, 0);
        let mut offset = KTX2_HEADER_SIZE + levels.len() * KTX2_LEVEL_ENTRY_SIZE;

        for level in &levels {
            for long in [offset, level.len(), level.len()] {
                bytes.extend_from_slice(&(long as u64).to_le_bytes());
            }
            offset += level.len();
        }

        for level in &levels {
            bytes.extend_from_slice(level);
        }

        std::fs::write(path, bytes).unwrap();
    }

    fn demand(finest_level: u32) -> MipDemand {
        MipDemand {
            finest_level,
            retention: Duration::ZERO,
        }
    }

    // Updates and records each frame's copies until the resident first level is `level`.
    fn run_mips_until(
        resources: &Resources,
        device: &TestDevice,
        handle: &MipHandle<CubeMap>,
        level: u32,
    ) -> Arc<CubeMap> {
        for _ in 0..MAX_FRAMES {
            resources.update().unwrap();
            device
                .run(|command_buffer| resources.record_mip_copies(command_buffer))
                .unwrap();

            if let Some(current) = handle.get()
                && current.first_level() == level
            {
                return current;
            }

            std::thread::sleep(Duration::from_millis(1));
        }

        panic!("the levels from {level} did not become resident in {MAX_FRAMES} frames");
    }

    #[test]
    fn mip_levels_stream_in_and_out_keeping_the_levels_they_share() {
        let _gpu = GPU.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let device = TestDevice::new().unwrap();
        let resources = Resources::new(device.memory(), device.transfer(), device.bindless());
        let path = std::env::temp_dir().join(format!("xsa-mips-{}.ktx2", std::process::id()));
        write_tagged_cube(&path);
        let handle = resources.load_mips::<CubeMap>(
            LoadRequest::new(&path, "tagged cube"),
            resource_loader::texture::ColorSpace::Linear,
        );
        handle.set_demand(demand(2));
        run_mips_until(&resources, &device, &handle, 2);
        handle.set_demand(demand(0));
        let finest = run_mips_until(&resources, &device, &handle, 0);
        assert_eq!(finest.image().subresource_range().level_count, CUBE_LEVELS);
        drop(finest);
        handle.set_demand(demand(1));
        let coarser = run_mips_until(&resources, &device, &handle, 1);
        let side = CUBE_SIDE >> 1;
        let to_copy_source = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_stage_mask(vk::PipelineStageFlags2::COPY)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .image(coarser.image().handle())
            .subresource_range(coarser.image().subresource_range());
        device
            .run(|command_buffer| unsafe {
                device.memory().device().cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&to_copy_source)),
                );
            })
            .unwrap();
        let bytes = device
            .read_region(
                coarser.image(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::Offset2D::default(),
                vk::Extent2D {
                    width: side,
                    height: side,
                },
                (side * side * TEXEL_SIZE) as usize,
            )
            .unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(bytes, vec![1; (side * side * TEXEL_SIZE) as usize]);
        assert_eq!(device.validation_messages(), Vec::<String>::new());
    }
}
