use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use memory_vulkan::{ImageCopy, ImageUpload, MemoryContext, RegionCopy, TransferPriority, TransferQueue};

use crate::streaming::{
    AnySet, CancelledUpload, CellAddress, CellContent, CellUse, ChunkId, MissingPage, PageFormat, PageMove, PageNeed,
    PageOwner, PagePool, PageRetention, PageSource, PageUpload, PoolCapacity, PoolShrink, ReadOutcome, ReadPage,
    Residency, SetId, SetState, StreamedSet, StreamingLog, Victim, VictimQueue,
};
use crate::{Admission, EvictionRank, LoadEnvironment, LoadPriority, MemoryUsage, SoftOwner, SoftUsage, WorkerPool};

// Keeps every streamed set's wanted pages resident as far as the budget allows: reads them on the worker pool, keeps
// them decoded in RAM, copies them into the cells of its pools' chunks, and evicts what matters least. A pool grows by
// a chunk when wanted pages find no cell and the budget admits it. It shrinks when the budget evicts its least
// valuable pages: once a chunk's worth of cells is free, the pages left in the chunk it releases move into them.
pub(crate) struct Streamer {
    sets: Vec<Arc<dyn AnySet>>,
    pools: HashMap<PageFormat, PagePool>,
    uploads: Vec<PageUpload>,
    moves: Vec<PageMove>,
    finished: Arc<Mutex<Vec<ReadPage>>>,
    reading: Arc<AtomicUsize>,
    closed: Arc<Mutex<Vec<SetId>>>,
    // Pools admitted to grow by a chunk.
    starved: HashSet<PageFormat>,
    next_set: u64,
    next_chunk: u64,
    log: StreamingLog,
}

// The cells of one pool the missing pages of an update may still claim, in score order.
struct CellClaims {
    free: usize,
    returning: usize,
    victims: VictimQueue,
    // Cells of chunks being added.
    growth: usize,
}

// How a missing page gets a cell.
enum CellClaim {
    Free,
    // Waits for a cell the GPU is still done with.
    Returning,
    // Takes the cell of a page that matters less.
    Victim(Victim),
    // Waits for a chunk being added.
    Growth,
    // Waits for a chunk the budget just admitted.
    Grow,
}

impl Streamer {
    pub fn new() -> Streamer {
        Streamer {
            sets: Vec::new(),
            pools: HashMap::new(),
            uploads: Vec::new(),
            moves: Vec::new(),
            finished: Arc::new(Mutex::new(Vec::new())),
            reading: Arc::new(AtomicUsize::new(0)),
            closed: Arc::new(Mutex::new(Vec::new())),
            starved: HashSet::new(),
            next_set: 0,
            next_chunk: 0,
            log: StreamingLog::new(),
        }
    }

    // # Panics
    //
    // If `format`'s pages are not a whole number of texel blocks: cells must stay aligned to blocks.
    pub fn register<S: PageSource>(&mut self, source: S, format: PageFormat, name: String) -> StreamedSet<S> {
        let block = format.texel_block.extent;
        assert!(
            format.extent.width.is_multiple_of(block.width) && format.extent.height.is_multiple_of(block.height),
            "{name}: pages of {:?} are not a whole number of {:?} texel blocks",
            format.extent,
            block
        );

        self.next_set += 1;
        let state = Arc::new(SetState::new(SetId { value: self.next_set }, name, format, source));
        self.sets.push(Arc::clone(&state) as Arc<dyn AnySet>);
        self.pools.entry(format).or_insert_with(|| PagePool::new(format));

        StreamedSet::new(state, Arc::clone(&self.closed))
    }

    // Runs once per update, `now` counting updates. At most `max_reads` pages read at once, and only what `admission`
    // admits loads. Returns the video memory of the chunks released.
    pub fn update(
        &mut self,
        now: u64,
        environment: &LoadEnvironment,
        workers: &WorkerPool,
        max_reads: usize,
        admission: &mut Admission,
    ) -> u64 {
        self.close_sets(&environment.memory);

        for pool in self.pools.values_mut() {
            pool.update(&environment.bindless);
        }

        self.finish_reads();
        self.finish_uploads(&environment.memory);
        self.finish_moves(&environment.memory);

        for set in self.live_sets() {
            let cancelled = set.apply(now);
            self.cancel_uploads(set.id(), &cancelled, &environment.memory);
        }

        self.start_moves(&environment.transfer);
        let released = self.release_drained();
        self.dispatch(now, environment, workers, max_reads, admission);
        self.log.update(&self.sets, &self.pools);

        released
    }

    // Adds a chunk to each pool admitted to grow.
    pub fn grow(&mut self, environment: &LoadEnvironment) {
        for format in &self.starved {
            let pool = self.pools.get_mut(format).expect("a starved pool exists");

            if pool.is_growing() {
                continue;
            }

            self.next_chunk += 1;

            if let Err(err) = pool.add_chunk(environment, ChunkId { value: self.next_chunk }) {
                tracing::warn!("adding a chunk of streamed pages failed: {err:#}");
            }
        }
    }

    // Every cell and page's bytes eviction may free, with what keeping each is worth. Pinned pages, moving pages,
    // frozen sets and the cells of shrinking pools are left out.
    pub fn soft_usage(&self, now: u64, transfer: &TransferQueue) -> Vec<SoftUsage> {
        let mut usage = Vec::new();

        for set in self.live_sets() {
            let format = set.format();
            let pool = &self.pools[&format];
            let upload_time = transfer.copy_duration(format.page_size() as u64);
            let cell_size = pool.cell_size();

            if !pool.is_draining() {
                usage.extend(Streamer::evictable_cells(&set).map(|cell_use| SoftUsage {
                    owner: SoftOwner::Cell {
                        pool: format,
                        page: PageOwner {
                            set: set.id(),
                            page: cell_use.page,
                        },
                    },
                    memory: MemoryUsage {
                        video: cell_size,
                        ram: 0,
                    },
                    rank: Streamer::cell_rank(&cell_use, now, upload_time, cell_size),
                }));
            }

            usage.extend(set.byte_uses().into_iter().map(|byte_use| SoftUsage {
                owner: SoftOwner::PageBytes(PageOwner {
                    set: set.id(),
                    page: byte_use.page,
                }),
                memory: MemoryUsage {
                    video: 0,
                    ram: byte_use.bytes,
                },
                rank: match byte_use.score {
                    Some(score) => EvictionRank::Wanted { score },
                    None => EvictionRank::Idle {
                        reload_time: byte_use.read_time,
                        bytes: byte_use.bytes,
                        idle_updates: now.saturating_sub(byte_use.last_use),
                    },
                },
            }));
        }

        usage
    }

    // The pools eviction may shrink.
    pub fn pool_capacities(&self) -> Vec<PoolCapacity> {
        self.pools.values().filter_map(PagePool::capacity).collect()
    }

    pub fn ram_bytes(&self) -> u64 {
        self.sets.iter().map(|set| set.ram_bytes()).sum()
    }

    // Starts releasing `shrink.chunks` chunks of a pool, those holding the fewest pages once `shrink.evicted` lost
    // their cells; the pages left in them move into the cells freed. Returns the video memory being released.
    pub fn shrink(&mut self, shrink: &PoolShrink, memory: &MemoryContext) -> u64 {
        let Some(pool) = self.pools.get_mut(&shrink.format) else {
            return 0;
        };

        if pool.is_draining() {
            return 0;
        }

        let sets: Vec<_> = self
            .sets
            .iter()
            .filter(|set| set.format() == shrink.format)
            .cloned()
            .collect();
        let find = |id: SetId| sets.iter().find(|set| set.id() == id);

        for owner in &shrink.evicted {
            if let Some(set) = find(owner.set) {
                for cell in set.evict_cell(owner.page) {
                    Streamer::give_back(pool, cell, memory);
                }

                self.uploads.retain(|upload| upload.owner != *owner);
            }
        }

        let mut held: HashMap<ChunkId, Vec<(SetId, CellUse)>> = HashMap::new();

        for set in &sets {
            for cell_use in set.cell_uses() {
                held.entry(cell_use.cell.chunk).or_default().push((set.id(), cell_use));
            }
        }

        let mut candidates: Vec<(ChunkId, u64)> = pool
            .chunks()
            .iter()
            .filter(|chunk| chunk.ready().is_some())
            .map(|chunk| (chunk.id, chunk.size))
            .collect();
        candidates.sort_by_key(|(chunk, _)| held.get(chunk).map_or(0, Vec::len));
        let mut releasing = 0;

        for (chunk, size) in candidates.into_iter().take(shrink.chunks) {
            pool.allocator.drain_chunk(chunk);
            releasing += size;

            for (set, cell_use) in held.remove(&chunk).unwrap_or_default() {
                let owner = PageOwner {
                    set,
                    page: cell_use.page,
                };

                match cell_use.content {
                    CellContent::Uploading => {
                        if let Some(set) = find(set) {
                            set.evict_cell(cell_use.page);
                        }

                        self.uploads.retain(|upload| upload.owner != owner);
                    }
                    CellContent::Resident | CellContent::Moving => {
                        pool.allocator.reserve(1);
                        pool.survivors.push(owner);
                    }
                }
            }
        }

        releasing
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

            for cell in set.release() {
                Streamer::give_back(pool, cell, memory);
            }

            self.uploads.retain(|upload| upload.owner.set != id);
            self.moves.retain(|page_move| page_move.owner.set != id);
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
                    let chunk = pool
                        .chunk(upload.cell.chunk)
                        .expect("an upload's chunk exists until the upload completes or is cancelled");
                    let (_, slot) = chunk.ready().expect("a chunk being written is ready");
                    set.finish_upload(
                        upload.owner.page,
                        upload.cell,
                        slot,
                        chunk.grid.offset(upload.cell.cell),
                    );
                    false
                }
                Err(err) => {
                    tracing::warn!("{}: uploading a page failed: {err}", set.name());

                    for cell in set.evict_cell(upload.owner.page) {
                        Streamer::give_back(pool, cell, memory);
                    }

                    false
                }
            }
        });

        self.uploads = uploads;
    }

    fn finish_moves(&mut self, memory: &MemoryContext) {
        let mut moves = std::mem::take(&mut self.moves);

        moves.retain_mut(|page_move| {
            let Some(set) = self.set(page_move.owner.set) else {
                return false;
            };
            let pool = &self.pools[&set.format()];

            match page_move.copy.poll() {
                Ok(None) => true,
                Ok(Some(_)) => {
                    let chunk = pool
                        .chunk(page_move.destination.chunk)
                        .expect("a move's destination chunk is not draining, so it stays");
                    let (_, slot) = chunk.ready().expect("a chunk being written is ready");
                    set.finish_move(
                        page_move.owner.page,
                        page_move.destination,
                        slot,
                        chunk.grid.offset(page_move.destination.cell),
                    );
                    false
                }
                Err(err) => {
                    tracing::warn!("{}: moving a page out of a released chunk failed: {err}", set.name());

                    for cell in set.evict_cell(page_move.owner.page) {
                        Streamer::give_back(pool, cell, memory);
                    }

                    false
                }
            }
        });

        self.moves = moves;
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
                .find(|pool| pool.chunk(cancelled.cell.chunk).is_some())
            {
                Streamer::give_back(pool, cancelled.cell, memory);
            }
        }
    }

    // Copies the pages left in draining chunks into the cells reserved for them, as those cells return. Pages that
    // left their cell since give their reservation back.
    fn start_moves(&mut self, transfer: &TransferQueue) {
        for pool in self.pools.values_mut() {
            let survivors = std::mem::take(&mut pool.survivors);

            for owner in survivors {
                let set = self.sets.iter().find(|set| set.id() == owner.set);
                let cell_use = set.and_then(|set| set.cell_use(owner.page)).filter(|cell_use| {
                    cell_use.content == CellContent::Resident && pool.allocator.is_draining(cell_use.cell.chunk)
                });
                let (Some(set), Some(cell_use)) = (set, cell_use) else {
                    pool.allocator.unreserve(1);
                    continue;
                };
                let Some(destination) = pool.allocator.take_reserved(cell_use.retention) else {
                    pool.survivors.push(owner);
                    continue;
                };
                let Some(source) = set.start_move(owner.page, destination) else {
                    pool.allocator.give_back(destination)();
                    continue;
                };
                let source_chunk = pool
                    .chunk(source.chunk)
                    .expect("a draining chunk stays until its pages left");
                let destination_chunk = pool
                    .chunk(destination.chunk)
                    .expect("a free cell is in a chunk of its pool");
                let (Some((source_image, _)), Some((destination_image, _))) =
                    (source_chunk.ready(), destination_chunk.ready())
                else {
                    unreachable!("a page moves between ready chunks");
                };
                let copy = RegionCopy {
                    source_offset: source_chunk.grid.offset(source.cell),
                    destination_offset: destination_chunk.grid.offset(destination.cell),
                    extent: pool.format.extent,
                };
                let ticket = transfer.copy_regions(
                    Arc::clone(source_image),
                    Arc::clone(destination_image),
                    &[copy],
                    pool.format.texel_block,
                    TransferPriority::High,
                );
                self.moves.push(PageMove {
                    owner,
                    destination,
                    copy: ticket.into(),
                });
            }
        }
    }

    // Releases the draining chunks no page holds any more; returns their video memory.
    fn release_drained(&mut self) -> u64 {
        let mut released = 0;

        for pool in self.pools.values_mut() {
            let draining: Vec<ChunkId> = pool.allocator.draining().collect();

            if draining.is_empty() {
                continue;
            }

            let held: HashSet<ChunkId> = self
                .sets
                .iter()
                .filter(|set| set.format() == pool.format)
                .flat_map(|set| set.cell_uses())
                .map(|cell_use| cell_use.cell.chunk)
                .collect();

            for chunk in draining.into_iter().filter(|chunk| !held.contains(chunk)) {
                released += pool.remove_chunk(chunk).map_or(0, |chunk| chunk.size);
            }
        }

        released
    }

    // Reads and places the missing pages of every live set from the highest score down. Each claims a cell first;
    // a page no cell would be found for is not read.
    fn dispatch(
        &mut self,
        now: u64,
        environment: &LoadEnvironment,
        workers: &WorkerPool,
        max_reads: usize,
        admission: &mut Admission,
    ) {
        let mut missing: Vec<(Arc<dyn AnySet>, MissingPage)> = self
            .live_sets()
            .into_iter()
            .flat_map(|set| set.missing().into_iter().map(move |page| (Arc::clone(&set), page)))
            .collect();
        missing.sort_by(|(_, a), (_, b)| b.score.total_cmp(&a.score));
        self.starved.clear();
        let mut claims: HashMap<PageFormat, CellClaims> = HashMap::new();
        let transfer = &environment.transfer;

        for (set, page) in missing {
            if page.need == PageNeed::Read && self.reading.load(Ordering::Acquire) >= max_reads {
                continue;
            }

            let format = set.format();

            if !claims.contains_key(&format) {
                let pool_claims = self.cell_claims(format, now, transfer);
                claims.insert(format, pool_claims);
            }

            let pool_claims = claims.get_mut(&format).expect("claims were just added");
            let Some(claim) = self.claim(format, page.score, pool_claims, admission, now, transfer) else {
                continue;
            };

            if matches!(claim, CellClaim::Grow) {
                self.starved.insert(format);
            }

            match (page.need, claim) {
                (PageNeed::Cell, CellClaim::Free) => {
                    let pool = self.pools.get_mut(&format).expect("a set's pool exists");
                    let cell = pool.take_cell(page.retention).expect("a claimed free cell is free");
                    self.upload(&set, page, cell, environment);
                }
                (PageNeed::Cell, CellClaim::Victim(victim)) => self.evict_victim(format, victim, &environment.memory),
                (PageNeed::Cell, CellClaim::Returning | CellClaim::Growth | CellClaim::Grow) => {}
                (PageNeed::Read, _) => {
                    let need = MemoryUsage {
                        video: 0,
                        ram: format.page_size() as u64,
                    };

                    if admission.admit(need, || self.idle_usage(now, transfer))
                        && let Some(job) = set.read_job(page.page, &self.finished, &self.reading)
                    {
                        workers.push(Streamer::priority(page.retention), job);
                    }
                }
            }
        }
    }

    fn cell_claims(&self, format: PageFormat, now: u64, transfer: &TransferQueue) -> CellClaims {
        let pool = &self.pools[&format];
        let growth = pool
            .chunks()
            .iter()
            .filter(|chunk| chunk.is_preparing())
            .map(|chunk| chunk.grid.cells() as usize)
            .sum();

        CellClaims {
            free: pool.allocator.available(),
            returning: pool.allocator.returning(),
            victims: VictimQueue::new(self.victims(format, now, transfer)),
            growth,
        }
    }

    // A cell for a page of `score`: a free one, one returning, one a page that matters less holds, one of a chunk
    // being added, or one of a chunk the budget admits now.
    fn claim(
        &self,
        format: PageFormat,
        score: f32,
        claims: &mut CellClaims,
        admission: &mut Admission,
        now: u64,
        transfer: &TransferQueue,
    ) -> Option<CellClaim> {
        if claims.free > 0 {
            claims.free -= 1;
            return Some(CellClaim::Free);
        }

        if claims.returning > 0 {
            claims.returning -= 1;
            return Some(CellClaim::Returning);
        }

        if let Some(victim) = claims.victims.take_for(score) {
            return Some(CellClaim::Victim(victim));
        }

        if claims.growth > 0 {
            claims.growth -= 1;
            return Some(CellClaim::Growth);
        }

        let pool = &self.pools[&format];

        if pool.is_growing() || pool.is_draining() || self.starved.contains(&format) {
            return None;
        }

        let need = MemoryUsage {
            video: pool.chunk_size(),
            ram: 0,
        };

        if !admission.admit(need, || self.idle_usage(now, transfer)) {
            return None;
        }

        claims.growth = pool.grid().cells() as usize - 1;

        Some(CellClaim::Grow)
    }

    fn evict_victim(&mut self, format: PageFormat, victim: Victim, memory: &MemoryContext) {
        let Some(set) = self.set(victim.owner.set) else {
            return;
        };

        for cell in set.evict_cell(victim.owner.page) {
            Streamer::give_back(&self.pools[&format], cell, memory);
        }

        self.uploads.retain(|upload| upload.owner != victim.owner);
    }

    fn upload(&mut self, set: &Arc<dyn AnySet>, page: MissingPage, cell: CellAddress, environment: &LoadEnvironment) {
        let pool = &self.pools[&set.format()];
        let Some(bytes) = set.start_upload(page.page, cell) else {
            pool.allocator.give_back(cell)();
            return;
        };
        let chunk = pool.chunk(cell.chunk).expect("a free cell is in a chunk of its pool");
        let (image, _) = chunk.ready().expect("a free cell is in a ready chunk");
        let format = pool.format;
        let upload = ImageUpload {
            data: vec![Box::new(bytes)],
            copies: vec![ImageCopy {
                data: 0,
                offset: 0,
                mip_level: 0,
                array_layer: 0,
                image_offset: chunk.grid.offset(cell.cell),
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
            cell,
            upload: ticket.into(),
        });
    }

    fn victims(&self, format: PageFormat, now: u64, transfer: &TransferQueue) -> Vec<Victim> {
        let pool = &self.pools[&format];
        let upload_time = transfer.copy_duration(format.page_size() as u64);
        let cell_size = pool.cell_size();

        self.live_sets()
            .into_iter()
            .filter(|set| set.format() == format)
            .flat_map(|set| {
                let id = set.id();

                Streamer::evictable_cells(&set)
                    .map(move |cell_use| Victim {
                        owner: PageOwner {
                            set: id,
                            page: cell_use.page,
                        },
                        rank: Streamer::cell_rank(&cell_use, now, upload_time, cell_size),
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    // What the budget may evict now: the streamed pages' idle usage, and the pools it may shrink.
    fn idle_usage(&self, now: u64, transfer: &TransferQueue) -> (Vec<SoftUsage>, Vec<PoolCapacity>) {
        let mut usage = self.soft_usage(now, transfer);
        usage.retain(|usage| usage.rank.is_idle());

        (usage, self.pool_capacities())
    }

    fn evictable_cells(set: &Arc<dyn AnySet>) -> impl Iterator<Item = CellUse> {
        set.cell_uses().into_iter().filter(|cell_use| {
            cell_use.retention == PageRetention::Evictable && cell_use.content != CellContent::Moving
        })
    }

    // Losing its cell costs an upload while its bytes stay in RAM, and a read as well otherwise.
    fn cell_rank(cell_use: &CellUse, now: u64, upload_time: std::time::Duration, cell_size: u64) -> EvictionRank {
        match cell_use.score {
            Some(score) => EvictionRank::Wanted { score },
            None => EvictionRank::Idle {
                reload_time: if cell_use.cached {
                    upload_time
                } else {
                    cell_use.read_time + upload_time
                },
                bytes: cell_size,
                idle_updates: now.saturating_sub(cell_use.last_use),
            },
        }
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

    // Returns `cell` to `pool` once the GPU finished every frame that may sample it and every copy that may write it.
    fn give_back(pool: &PagePool, cell: CellAddress, memory: &MemoryContext) {
        let give_back = pool.allocator.give_back(cell);
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
            .filter_map(|chunk| chunk.ready())
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
