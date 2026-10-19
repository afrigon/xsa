use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use ash::vk;
use memory_vulkan::{ImageCopy, ImageUpload, MemoryContext};

use crate::streaming::{
    AnySet, CancelledUpload, Chunk, ChunkId, LayerAddress, MissingPage, PageFormat, PageNeed, PageOwner, PagePool,
    PageRetention, PageSource, PageUpload, ReadOutcome, ReadPage, Residency, SetId, SetState, StreamedSet,
    StreamingLog, Victim, VictimQueue,
};
use crate::{LoadEnvironment, LoadPriority, MemoryUsage, SoftOwner, SoftUsage, WorkerPool};

// Keeps every streamed set's wanted pages resident as far as the budget allows: reads them on the worker pool, keeps
// them decoded in RAM, copies them into the layers of its pools' chunks, and evicts what matters least. Pools grow by
// a chunk when wanted pages find no layer and the budget has room, and shrink when the budget evicts a chunk.
pub(crate) struct Streamer {
    sets: Vec<Arc<dyn AnySet>>,
    pools: HashMap<PageFormat, PagePool>,
    uploads: Vec<PageUpload>,
    finished: Arc<Mutex<Vec<ReadPage>>>,
    reading: Arc<AtomicUsize>,
    closed: Arc<Mutex<Vec<SetId>>>,
    // Pools with wanted pages no layer can hold.
    starved: HashSet<PageFormat>,
    next_set: u64,
    next_chunk: u64,
    log: StreamingLog,
}

impl Streamer {
    pub fn new() -> Streamer {
        Streamer {
            sets: Vec::new(),
            pools: HashMap::new(),
            uploads: Vec::new(),
            finished: Arc::new(Mutex::new(Vec::new())),
            reading: Arc::new(AtomicUsize::new(0)),
            closed: Arc::new(Mutex::new(Vec::new())),
            starved: HashSet::new(),
            next_set: 0,
            next_chunk: 0,
            log: StreamingLog::new(),
        }
    }

    pub fn register<S: PageSource>(&mut self, source: S, format: PageFormat, name: String) -> StreamedSet<S> {
        self.next_set += 1;
        let state = Arc::new(SetState::new(SetId { value: self.next_set }, name, format, source));
        self.sets.push(Arc::clone(&state) as Arc<dyn AnySet>);
        self.pools.entry(format).or_insert_with(|| PagePool::new(format));

        StreamedSet::new(state, Arc::clone(&self.closed))
    }

    // Runs once per update, `now` counting updates. At most `max_reads` pages read at once.
    pub fn update(&mut self, now: u64, environment: &LoadEnvironment, workers: &WorkerPool, max_reads: usize) {
        self.close_sets(&environment.memory);

        for pool in self.pools.values_mut() {
            pool.update(&environment.bindless);
        }

        self.finish_reads();
        self.finish_uploads(&environment.memory);

        for set in self.live_sets() {
            let cancelled = set.apply(now);
            self.cancel_uploads(set.id(), &cancelled, &environment.memory);
        }

        self.dispatch(environment, workers, max_reads);
        self.log.update(&self.sets, &self.pools);
    }

    pub fn is_starved(&self) -> bool {
        !self.starved.is_empty()
    }

    // Adds a chunk to each starved pool whose next chunk fits in `room` bytes of video memory; returns whether any
    // was added.
    pub fn grow(&mut self, mut room: u64, now: u64, environment: &LoadEnvironment) -> bool {
        let mut grew = false;

        for format in &self.starved {
            let pool = self.pools.get_mut(format).expect("a starved pool exists");
            let size = pool.chunk_size();

            if pool.is_growing() || size > room {
                continue;
            }

            self.next_chunk += 1;

            match pool.add_chunk(environment, ChunkId { value: self.next_chunk }, now) {
                Ok(()) => {
                    room -= size;
                    grew = true;
                }
                Err(err) => tracing::warn!("adding a chunk of streamed pages failed: {err:#}"),
            }
        }

        grew
    }

    // Every chunk eviction may release, with its most recent use and the score of its wanted pages. Chunks holding
    // pinned pages or pages of a frozen set are never released.
    pub fn chunk_usage(&self) -> Vec<SoftUsage> {
        let mut usage: HashMap<ChunkId, SoftUsage> = HashMap::new();
        let mut protected = HashSet::new();

        for chunk in self.pools.values().flat_map(PagePool::chunks) {
            usage.insert(
                chunk.id,
                SoftUsage {
                    owner: SoftOwner::Chunk(chunk.id),
                    memory: MemoryUsage {
                        video: chunk.size,
                        ram: 0,
                    },
                    last_use: chunk.created_in,
                    score: 0.0,
                },
            );
        }

        for set in &self.sets {
            let frozen = set.residency() == Residency::Frozen;

            for layer_use in set.layer_uses() {
                if frozen || layer_use.retention == PageRetention::Pinned {
                    protected.insert(layer_use.layer.chunk);
                }

                if let Some(chunk) = usage.get_mut(&layer_use.layer.chunk) {
                    chunk.last_use = chunk.last_use.max(layer_use.last_use);
                    chunk.score += layer_use.score.unwrap_or(0.0);
                }
            }
        }

        usage
            .into_iter()
            .filter(|(chunk, _)| !protected.contains(chunk))
            .map(|(_, usage)| usage)
            .collect()
    }

    // Every page's decoded bytes eviction may drop, except those of frozen sets.
    pub fn page_bytes_usage(&self) -> Vec<SoftUsage> {
        self.sets
            .iter()
            .filter(|set| set.residency() == Residency::Live)
            .flat_map(|set| {
                let id = set.id();

                set.byte_uses().into_iter().map(move |byte_use| SoftUsage {
                    owner: SoftOwner::PageBytes(PageOwner {
                        set: id,
                        page: byte_use.page,
                    }),
                    memory: MemoryUsage {
                        video: 0,
                        ram: byte_use.bytes,
                    },
                    last_use: byte_use.last_use,
                    score: byte_use.score.unwrap_or(0.0),
                })
            })
            .collect()
    }

    pub fn ram_bytes(&self) -> u64 {
        self.sets.iter().map(|set| set.ram_bytes()).sum()
    }

    // Releases a chunk with every page in it, without copying: its wanted pages fall back to coarser pages until
    // they find another layer.
    pub fn release_chunk(&mut self, chunk: ChunkId) -> Option<Chunk> {
        let pool = self.pools.values_mut().find(|pool| pool.chunk(chunk).is_some())?;
        let format = pool.format;

        for set in self.sets.iter().filter(|set| set.format() == format) {
            for cancelled in set.evict_chunk(chunk) {
                let owner = PageOwner {
                    set: set.id(),
                    page: cancelled.page,
                };
                self.uploads.retain(|upload| upload.owner != owner);
            }
        }

        pool.remove_chunk(chunk)
    }

    pub fn evict_page_bytes(&self, owner: PageOwner) {
        if let Some(set) = self.set(owner.set) {
            set.evict_bytes(owner.page);
        }
    }

    pub fn closed(&self) -> MutexGuard<'_, Vec<SetId>> {
        self.closed.lock().expect("the closed sets' lock is poisoned")
    }

    fn close_sets(&mut self, memory: &MemoryContext) {
        let closed = std::mem::take(&mut *self.closed());

        for id in closed {
            let Some(index) = self.sets.iter().position(|set| set.id() == id) else {
                continue;
            };
            let set = self.sets.remove(index);
            let pool = &self.pools[&set.format()];

            for layer in set.release() {
                Streamer::give_back(pool, layer, memory);
            }

            self.uploads.retain(|upload| upload.owner.set != id);
        }
    }

    fn finish_reads(&mut self) {
        let finished = std::mem::take(&mut *self.finished.lock().expect("the finished reads' lock is poisoned"));

        for read in finished {
            let Some(set) = self.set(read.owner.set) else {
                continue;
            };

            if let ReadOutcome::Failed(err) = set.finish_read(read.owner.page, read.result) {
                tracing::warn!("{}: reading a page failed: {err:#}", set.name());
            }
        }
    }

    fn finish_uploads(&mut self, memory: &MemoryContext) {
        let mut uploads = std::mem::take(&mut self.uploads);

        uploads.retain_mut(|upload| {
            let Some(set) = self.set(upload.owner.set) else {
                return false;
            };
            let pool = &self.pools[&set.format()];

            match upload.upload.poll() {
                Ok(None) => true,
                Ok(Some(_)) => {
                    let (_, slot) = pool
                        .chunk(upload.layer.chunk)
                        .and_then(Chunk::ready)
                        .expect("an upload's chunk is ready until the upload completes or is cancelled");
                    set.finish_upload(upload.owner.page, upload.layer, slot);
                    false
                }
                Err(err) => {
                    tracing::warn!("{}: uploading a page failed: {err}", set.name());

                    if let Some(layer) = set.evict_layer(upload.owner.page) {
                        Streamer::give_back(pool, layer, memory);
                    }

                    false
                }
            }
        });

        self.uploads = uploads;
    }

    fn cancel_uploads(&mut self, set: SetId, cancelled: &[CancelledUpload], memory: &MemoryContext) {
        for cancelled in cancelled {
            let owner = PageOwner {
                set,
                page: cancelled.page,
            };
            self.uploads.retain(|upload| upload.owner != owner);

            if let Some(pool) = self
                .pools
                .values()
                .find(|pool| pool.chunk(cancelled.layer.chunk).is_some())
            {
                Streamer::give_back(pool, cancelled.layer, memory);
            }
        }
    }

    // Reads and places the missing pages of every live set from the highest score down.
    fn dispatch(&mut self, environment: &LoadEnvironment, workers: &WorkerPool, max_reads: usize) {
        let mut missing: Vec<(Arc<dyn AnySet>, MissingPage)> = self
            .live_sets()
            .into_iter()
            .flat_map(|set| set.missing().into_iter().map(move |page| (Arc::clone(&set), page)))
            .collect();
        missing.sort_by(|(_, a), (_, b)| b.score.total_cmp(&a.score));
        self.starved.clear();
        let mut victims: HashMap<PageFormat, VictimQueue> = HashMap::new();
        let mut returning: HashMap<PageFormat, usize> = HashMap::new();

        for (set, page) in missing {
            match page.need {
                PageNeed::Read => {
                    if self.reading.load(Ordering::Acquire) >= max_reads {
                        continue;
                    }

                    if let Some(job) = set.read_job(page.page, &self.finished, &self.reading) {
                        workers.push(Streamer::priority(page.retention), job);
                    }
                }
                PageNeed::Layer => self.place(&set, page, environment, &mut victims, &mut returning),
            }
        }
    }

    // Finds `page` a free layer, or makes one return by evicting a page that matters less.
    fn place(
        &mut self,
        set: &Arc<dyn AnySet>,
        page: MissingPage,
        environment: &LoadEnvironment,
        victims: &mut HashMap<PageFormat, VictimQueue>,
        returning: &mut HashMap<PageFormat, usize>,
    ) {
        let format = set.format();
        let pool = self.pools.get_mut(&format).expect("a set's pool exists");

        if let Some(layer) = pool.take_layer(page.retention) {
            self.upload(set, page, layer, environment);
            return;
        }

        let waiting = returning.entry(format).or_insert_with(|| pool.allocator.returning());

        if *waiting > 0 {
            *waiting -= 1;
            return;
        }

        if !victims.contains_key(&format) {
            victims.insert(format, VictimQueue::new(self.victims(format)));
        }

        let Some(victim) = victims.get_mut(&format).and_then(|queue| queue.take_for(page.score)) else {
            self.starved.insert(format);
            return;
        };

        if let Some(victim_set) = self.set(victim.owner.set)
            && let Some(layer) = victim_set.evict_layer(victim.owner.page)
        {
            self.uploads.retain(|upload| upload.owner != victim.owner);
            Streamer::give_back(&self.pools[&format], layer, &environment.memory);
        }
    }

    fn upload(&mut self, set: &Arc<dyn AnySet>, page: MissingPage, layer: LayerAddress, environment: &LoadEnvironment) {
        let pool = &self.pools[&set.format()];
        let Some(bytes) = set.start_upload(page.page, layer) else {
            pool.allocator.give_back(layer)();
            return;
        };
        let (image, _) = pool
            .chunk(layer.chunk)
            .and_then(Chunk::ready)
            .expect("a free layer is in a ready chunk");
        let format = pool.format;
        let upload = ImageUpload {
            data: vec![Box::new(bytes)],
            copies: vec![ImageCopy {
                data: 0,
                offset: 0,
                mip_level: 0,
                array_layer: layer.layer,
                image_offset: vk::Offset2D::default(),
                extent: format.extent,
            }],
            texel_block: format.texel_block,
        };
        let ticket = environment.transfer.upload_region(
            Arc::clone(image),
            upload,
            Streamer::priority(page.retention).transfer(),
        );
        self.uploads.push(PageUpload {
            owner: PageOwner {
                set: set.id(),
                page: page.page,
            },
            layer,
            upload: ticket.into(),
        });
    }

    fn victims(&self, format: PageFormat) -> Vec<Victim> {
        self.live_sets()
            .into_iter()
            .filter(|set| set.format() == format)
            .flat_map(|set| {
                let id = set.id();

                set.layer_uses()
                    .into_iter()
                    .filter(|layer_use| layer_use.retention == PageRetention::Evictable)
                    .map(move |layer_use| Victim {
                        owner: PageOwner {
                            set: id,
                            page: layer_use.page,
                        },
                        last_use: layer_use.last_use,
                        score: layer_use.score,
                    })
            })
            .collect()
    }

    fn live_sets(&self) -> Vec<Arc<dyn AnySet>> {
        self.sets
            .iter()
            .filter(|set| set.residency() == Residency::Live)
            .cloned()
            .collect()
    }

    fn set(&self, id: SetId) -> Option<&Arc<dyn AnySet>> {
        self.sets.iter().find(|set| set.id() == id)
    }

    // Returns `layer` to `pool` once the GPU finished every frame that may sample it and every copy that may write it.
    fn give_back(pool: &PagePool, layer: LayerAddress, memory: &MemoryContext) {
        let give_back = pool.allocator.give_back(layer);
        memory.defer_action(move |_| give_back());
    }

    fn priority(retention: PageRetention) -> LoadPriority {
        match retention {
            PageRetention::Evictable => LoadPriority::Normal,
            PageRetention::Pinned => LoadPriority::High,
        }
    }

    // The image of the chunk shaders read at `slot`.
    #[cfg(test)]
    pub fn chunk_image(&self, slot: u32) -> Option<Arc<memory_vulkan::Image>> {
        self.pools
            .values()
            .flat_map(PagePool::chunks)
            .filter_map(Chunk::ready)
            .find(|(_, chunk_slot)| *chunk_slot == slot)
            .map(|(image, _)| Arc::clone(image))
    }

    #[cfg(test)]
    pub fn video_bytes(&self) -> u64 {
        self.pools.values().map(PagePool::video_bytes).sum()
    }

    #[cfg(test)]
    pub fn chunk_count(&self) -> usize {
        self.pools.values().map(|pool| pool.chunks().len()).sum()
    }
}
