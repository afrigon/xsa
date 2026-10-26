use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Instant;

use ash::vk;
use memory_vulkan::{BindlessTable, Image};

use crate::{
    AnyEntry, AnyMipEntry, EntryId, EntryStatus, Events, Job, JobOwner, LoadContext, LoadEnvironment, LoadPriority,
    LoadRequest, MemoryUsage, MipDemand, PendingResource, Registry, ResourceKey, ResourceState, SoftOwner, SoftUsage,
    StreamedMips, Unavailable, WorkerPool,
};

// Each level of a 2D image holds a quarter of the texels of the one above it.
const LEVEL_SHRINK: f64 = 4.0;

// One mip-streamed resource of kind `K`: its status, the levels resident now and the reload under way.
pub(crate) struct MipEntry<K: StreamedMips> {
    status: EntryStatus,
    options: K::Options,
    residency: Mutex<Residency<K>>,
    demands: Mutex<Vec<Weak<Mutex<Option<MipDemand>>>>>,
}

struct Residency<K> {
    current: Option<Arc<K>>,
    reload: Reload<K>,
    // Since when the resident levels are finer than the holders want.
    unwanted_since: Option<UnwantedSince>,
    // The budget asked for the retained levels to go.
    releasing: bool,
    // A reload failed: the current levels stay, and nothing reloads again.
    failed: bool,
}

enum Reload<K> {
    Idle,
    Reading,
    Uploading { pending: Box<dyn PendingResource<K>> },
    // Uploaded, waiting for a frame to copy the levels it shares with the current value.
    Copying { value: K },
}

#[derive(Clone, Copy)]
struct UnwantedSince {
    update: u64,
    instant: Instant,
}

impl<K: StreamedMips> MipEntry<K> {
    pub fn new(
        id: EntryId,
        request: LoadRequest,
        options: K::Options,
        size: u64,
        events: Arc<Mutex<Events>>,
    ) -> MipEntry<K> {
        MipEntry {
            status: EntryStatus::new(id, request, size, events),
            options,
            residency: Mutex::new(Residency {
                current: None,
                reload: Reload::Idle,
                unwanted_since: None,
                releasing: false,
                failed: false,
            }),
            demands: Mutex::new(Vec::new()),
        }
    }

    pub fn key(&self) -> ResourceKey<K::Options> {
        ResourceKey {
            path: self.status.path().to_path_buf(),
            options: self.options.clone(),
        }
    }

    pub fn current(&self) -> Option<Arc<K>> {
        self.residency().current.clone()
    }

    // Whether the resident levels reach `demand`'s finest, with nothing reloading; any levels once nothing is demanded.
    pub fn is_settled(&self, demand: Option<MipDemand>) -> bool {
        let residency = self.residency();
        let Some(current) = residency.current.as_deref() else {
            return false;
        };

        matches!(residency.reload, Reload::Idle) && current.first_level() <= MipEntry::wanted_level(demand, current)
    }

    pub fn add_demand(&self, demand: &Arc<Mutex<Option<MipDemand>>>) {
        self.demands().push(Arc::downgrade(demand));
    }

    pub fn join(&self, request: &LoadRequest) {
        if self.status.join(request) {
            self.status.store_priority(request.priority);
        }
    }

    pub fn settle_unavailable(&self, unavailable: Unavailable) {
        self.status.settle_unavailable(unavailable);
    }

    // What every live handle demands together; None when none states a demand.
    fn demand(&self) -> Option<MipDemand> {
        let mut demands = self.demands();
        demands.retain(|demand| demand.strong_count() > 0);

        demands
            .iter()
            .filter_map(|demand| *demand.upgrade()?.lock().expect("a mip demand's lock is poisoned"))
            .reduce(MipDemand::combined)
    }

    // The resident level the demand asks for; the smallest until the first levels are known.
    fn wanted_level(demand: Option<MipDemand>, current: &K) -> u32 {
        demand
            .map_or(u32::MAX, |demand| demand.finest_level)
            .min(current.mip_levels() - 1)
    }

    fn start_reload(
        self: &Arc<Self>,
        residency: &mut Residency<K>,
        first_level: u32,
        environment: &Arc<LoadEnvironment>,
        uploading: &Arc<Mutex<Vec<Arc<dyn AnyEntry>>>>,
        pool: &WorkerPool,
    ) {
        residency.reload = Reload::Reading;
        residency.releasing = false;
        let entry = Arc::clone(self);
        let current = residency.current.clone();
        let environment = Arc::clone(environment);
        let uploading = Arc::clone(uploading);
        pool.push(
            self.status.priority(),
            Job {
                owner: JobOwner::Entry(self.status.id()),
                run: Box::new(move || entry.reload(&environment, &uploading, first_level, current.as_deref())),
            },
        );
    }

    // On a worker: reads the levels the current value lacks and queues their upload.
    fn reload(
        self: &Arc<Self>,
        environment: &LoadEnvironment,
        uploading: &Mutex<Vec<Arc<dyn AnyEntry>>>,
        first_level: u32,
        current: Option<&K>,
    ) {
        if self.status.is_forgotten() {
            return;
        }

        let priority: LoadPriority = self.status.priority();
        let context = LoadContext {
            memory: &environment.memory,
            transfer: &environment.transfer,
            bindless: &environment.bindless,
            name: self.status.requester(),
            priority: priority.transfer(),
        };

        match K::load(&context, self.status.path(), &self.options, first_level, current) {
            Ok(pending) => {
                self.residency().reload = Reload::Uploading { pending };
                uploading
                    .lock()
                    .expect("the uploading resources' lock is poisoned")
                    .push(Arc::clone(self) as Arc<dyn AnyEntry>);
            }
            Err(err) => self.fail(err),
        }
    }

    // A first load that fails leaves the resource unavailable; a later one keeps the levels already resident.
    fn fail(&self, error: anyhow::Error) {
        let mut residency = self.residency();
        residency.reload = Reload::Idle;

        if residency.current.is_none() {
            drop(residency);
            self.settle_unavailable(Unavailable::Failed(Arc::new(error)));
            return;
        }

        residency.failed = true;
        tracing::warn!("{}: reloading its mip levels: {error:#}", self.status.describe());
    }

    fn residency(&self) -> MutexGuard<'_, Residency<K>> {
        self.residency.lock().expect("a mip entry's lock is poisoned")
    }

    fn demands(&self) -> MutexGuard<'_, Vec<Weak<Mutex<Option<MipDemand>>>>> {
        self.demands.lock().expect("a mip entry's demands' lock is poisoned")
    }

    // Moves `levels` of `image`, all its layers, from one layout to another.
    fn barrier(
        image: &Image,
        levels: std::ops::Range<u32>,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> vk::ImageMemoryBarrier2<'static> {
        let (source_access, destination_access) = match new_layout {
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                (vk::AccessFlags2::TRANSFER_WRITE, vk::AccessFlags2::SHADER_READ)
            }
            _ => (
                vk::AccessFlags2::SHADER_READ,
                vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE,
            ),
        };
        let (source_stage, destination_stage) = match new_layout {
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                (vk::PipelineStageFlags2::COPY, vk::PipelineStageFlags2::ALL_COMMANDS)
            }
            _ => (vk::PipelineStageFlags2::ALL_COMMANDS, vk::PipelineStageFlags2::COPY),
        };

        vk::ImageMemoryBarrier2::default()
            .src_stage_mask(source_stage)
            .src_access_mask(source_access)
            .dst_stage_mask(destination_stage)
            .dst_access_mask(destination_access)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(image.handle())
            .subresource_range(
                image
                    .subresource_range()
                    .base_mip_level(levels.start)
                    .level_count(levels.end - levels.start),
            )
    }
}

impl<K: StreamedMips> AnyEntry for MipEntry<K> {
    fn id(&self) -> EntryId {
        self.status.id()
    }

    fn size(&self) -> u64 {
        self.status.size()
    }

    fn state(&self) -> ResourceState {
        self.status.state(self.residency().current.is_some())
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
        let residency = self.residency();
        let current = residency.current.as_deref().map_or(MemoryUsage::default(), K::memory);

        match &residency.reload {
            Reload::Copying { value } => current + value.memory(),
            _ => current,
        }
    }

    fn poll(&self, bindless: &BindlessTable) -> bool {
        let mut residency = self.residency();
        let Reload::Uploading { pending } = &mut residency.reload else {
            return true;
        };

        match pending.poll(bindless) {
            Ok(None) => false,
            Ok(Some(value)) => {
                if residency.current.is_none() {
                    residency.current = Some(Arc::new(value));
                    residency.reload = Reload::Idle;
                } else {
                    residency.reload = Reload::Copying { value };
                }

                true
            }
            Err(err) => {
                drop(residency);
                self.fail(err);
                true
            }
        }
    }

    fn set_priority(&self, priority: LoadPriority) {
        self.status.store_priority(priority);

        if let Reload::Uploading { pending } = &self.residency().reload {
            pending.set_priority(priority.transfer());
        }
    }

    fn forget(&self, registry: &mut Registry) {
        self.status.mark_forgotten();
        registry.mip_kind::<K>().remove(&self.key());
        registry.entries.remove(&self.id());
        registry.mip_entries.remove(&self.id());
    }

    fn describe(&self) -> String {
        self.status.describe()
    }
}

impl<K: StreamedMips> AnyMipEntry for MipEntry<K> {
    fn update_residency(
        self: Arc<Self>,
        update: u64,
        now: Instant,
        environment: &Arc<LoadEnvironment>,
        uploading: &Arc<Mutex<Vec<Arc<dyn AnyEntry>>>>,
        pool: &WorkerPool,
    ) {
        let demand = self.demand();
        let mut residency = self.residency();

        if residency.failed || !matches!(residency.reload, Reload::Idle) || self.status.is_forgotten() {
            return;
        }

        let Some(current) = residency.current.clone() else {
            if matches!(self.status.state(false), ResourceState::Loading) {
                let first_level = demand.map_or(u32::MAX, |demand| demand.finest_level);
                self.start_reload(&mut residency, first_level, environment, uploading, pool);
            }

            return;
        };
        let wanted = MipEntry::wanted_level(demand, current.as_ref());
        let resident = current.first_level();

        if wanted == resident {
            residency.unwanted_since = None;
            return;
        }

        if wanted < resident {
            residency.unwanted_since = None;
            self.start_reload(&mut residency, wanted, environment, uploading, pool);
            return;
        }

        let since = *residency
            .unwanted_since
            .get_or_insert(UnwantedSince { update, instant: now });
        let retention = demand.map_or(std::time::Duration::ZERO, |demand| demand.retention);

        if residency.releasing || now.duration_since(since.instant) >= retention {
            self.start_reload(&mut residency, wanted, environment, uploading, pool);
        }
    }

    fn record_copies(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        let mut residency = self.residency();
        let Reload::Copying { .. } = residency.reload else {
            return;
        };
        let Reload::Copying { value } = std::mem::replace(&mut residency.reload, Reload::Idle) else {
            unreachable!("the reload was just seen copying");
        };
        let current = residency
            .current
            .take()
            .expect("a reload copies only once levels are resident");
        let shared_first = current.first_level().max(value.first_level());
        let source_levels = shared_first - current.first_level()..value.mip_levels() - current.first_level();
        let destination_levels = shared_first - value.first_level()..value.mip_levels() - value.first_level();
        let (source, destination) = (current.image(), value.image());
        let layers = source.subresource_range().layer_count;
        let extent = destination.extent();
        let regions: Vec<vk::ImageCopy> = destination_levels
            .clone()
            .zip(source_levels.clone())
            .map(|(destination_level, source_level)| {
                let subresource = |level| {
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(destination.subresource_range().aspect_mask)
                        .mip_level(level)
                        .layer_count(layers)
                };

                vk::ImageCopy::default()
                    .src_subresource(subresource(source_level))
                    .dst_subresource(subresource(destination_level))
                    .extent(vk::Extent3D {
                        width: (extent.width >> destination_level).max(1),
                        height: (extent.height >> destination_level).max(1),
                        depth: 1,
                    })
            })
            .collect();
        let before = [
            MipEntry::<K>::barrier(
                source,
                source_levels.clone(),
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ),
            MipEntry::<K>::barrier(
                destination,
                destination_levels.clone(),
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ),
        ];
        let after = [
            MipEntry::<K>::barrier(
                source,
                source_levels,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ),
            MipEntry::<K>::barrier(
                destination,
                destination_levels,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ),
        ];

        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&before),
            );
            device.cmd_copy_image(
                command_buffer,
                source.handle(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                destination.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&after),
            );
        }

        residency.current = Some(Arc::new(value));
    }

    fn retained_usage(&self) -> Option<SoftUsage> {
        let demand = self.demand();
        let residency = self.residency();
        let since = residency.unwanted_since?;
        let current = residency.current.as_deref()?;
        let shrink = MipEntry::wanted_level(demand, current) - current.first_level();
        let retained = 1.0 - LEVEL_SHRINK.powi(-(shrink as i32));
        let memory = current.memory();

        Some(SoftUsage {
            owner: SoftOwner::RetainedLevels(self.status.id()),
            memory: MemoryUsage {
                video: (memory.video as f64 * retained) as u64,
                ram: (memory.ram as f64 * retained) as u64,
            },
            last_use: since.update,
            score: 0.0,
        })
    }

    fn release_retained(&self) {
        self.residency().releasing = true;
    }
}
