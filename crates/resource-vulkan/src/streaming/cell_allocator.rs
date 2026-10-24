use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::streaming::{CellAddress, ChunkId, PageRetention};

// The free cells of a pool's ready chunks. A cell given back returns only once the action it hands out has run:
// the streamer defers it until the GPU finished every frame that may still sample the cell and every copy that may
// still write it. Pinned pages take the lowest free cell and others the highest, so pinned pages gather in few
// chunks. A draining chunk hands out no cell, and the cells reserved for the pages moving out of it are taken only
// by those pages.
pub(crate) struct CellAllocator {
    chunks: BTreeMap<ChunkId, u32>,
    draining: BTreeSet<ChunkId>,
    free: BTreeSet<CellAddress>,
    reserved: usize,
    returned: Arc<Mutex<Vec<CellAddress>>>,
    // Cells given back that have not returned, by chunk.
    returning: Arc<Mutex<HashMap<ChunkId, usize>>>,
}

impl CellAllocator {
    pub fn new() -> CellAllocator {
        CellAllocator {
            chunks: BTreeMap::new(),
            draining: BTreeSet::new(),
            free: BTreeSet::new(),
            reserved: 0,
            returned: Arc::new(Mutex::new(Vec::new())),
            returning: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_chunk(&mut self, chunk: ChunkId, cells: u32) {
        self.chunks.insert(chunk, cells);
        self.free.extend((0..cells).map(|cell| CellAddress { chunk, cell }));
    }

    pub fn remove_chunk(&mut self, chunk: ChunkId) {
        self.chunks.remove(&chunk);
        self.draining.remove(&chunk);
        self.free.retain(|cell| cell.chunk != chunk);
    }

    // Stops handing out `chunk`'s cells, until it is removed.
    pub fn drain_chunk(&mut self, chunk: ChunkId) {
        self.draining.insert(chunk);
        self.free.retain(|cell| cell.chunk != chunk);
    }

    pub fn is_draining(&self, chunk: ChunkId) -> bool {
        self.draining.contains(&chunk)
    }

    pub fn draining(&self) -> impl Iterator<Item = ChunkId> + '_ {
        self.draining.iter().copied()
    }

    pub fn take(&mut self, retention: PageRetention) -> Option<CellAddress> {
        if self.free.len() <= self.reserved {
            return None;
        }

        self.pop(retention)
    }

    // Sets `cells` free cells, present or returning, aside for `take_reserved`.
    pub fn reserve(&mut self, cells: usize) {
        self.reserved += cells;
    }

    pub fn unreserve(&mut self, cells: usize) {
        self.reserved -= cells;
    }

    pub fn take_reserved(&mut self, retention: PageRetention) -> Option<CellAddress> {
        let cell = self.pop(retention)?;
        self.reserved -= 1;

        Some(cell)
    }

    // What returns `cell` once run.
    pub fn give_back(&self, cell: CellAddress) -> impl FnOnce() + Send + 'static {
        *CellAllocator::lock(&self.returning).entry(cell.chunk).or_default() += 1;
        let returned = Arc::clone(&self.returned);
        let returning = Arc::clone(&self.returning);

        move || {
            CellAllocator::lock(&returned).push(cell);
            let mut returning = CellAllocator::lock(&returning);

            if let Some(count) = returning.get_mut(&cell.chunk) {
                *count -= 1;

                if *count == 0 {
                    returning.remove(&cell.chunk);
                }
            }
        }
    }

    // Frees the cells returned since the last call; those of released and draining chunks are dropped.
    pub fn collect_returned(&mut self) {
        let returned = std::mem::take(&mut *CellAllocator::lock(&self.returned));
        let chunks = &self.chunks;
        let draining = &self.draining;
        self.free.extend(
            returned
                .into_iter()
                .filter(|cell| chunks.contains_key(&cell.chunk) && !draining.contains(&cell.chunk)),
        );
    }

    // Free cells not reserved.
    pub fn available(&self) -> usize {
        self.free.len().saturating_sub(self.reserved)
    }

    // Cells given back to chunks still handing out cells that have not returned yet.
    pub fn returning(&self) -> usize {
        CellAllocator::lock(&self.returning)
            .iter()
            .filter(|(chunk, _)| self.chunks.contains_key(chunk) && !self.draining.contains(chunk))
            .map(|(_, count)| count)
            .sum()
    }

    pub fn capacity(&self) -> u64 {
        self.chunks.values().map(|&cells| cells as u64).sum()
    }

    fn pop(&mut self, retention: PageRetention) -> Option<CellAddress> {
        match retention {
            PageRetention::Pinned => self.free.pop_first(),
            PageRetention::Evictable => self.free.pop_last(),
        }
    }

    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().expect("a cell allocator's lock is poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(value: u64) -> ChunkId {
        ChunkId { value }
    }

    #[test]
    fn pinned_pages_take_the_lowest_cells_and_others_the_highest() {
        let mut allocator = CellAllocator::new();
        allocator.add_chunk(chunk(1), 2);
        allocator.add_chunk(chunk(2), 2);

        assert_eq!(
            allocator.take(PageRetention::Pinned),
            Some(CellAddress {
                chunk: chunk(1),
                cell: 0
            })
        );
        assert_eq!(
            allocator.take(PageRetention::Evictable),
            Some(CellAddress {
                chunk: chunk(2),
                cell: 1
            })
        );
    }

    #[test]
    fn a_cell_given_back_is_free_only_once_its_return_runs() {
        let mut allocator = CellAllocator::new();
        allocator.add_chunk(chunk(1), 1);
        let cell = allocator.take(PageRetention::Evictable).unwrap();
        let give_back = allocator.give_back(cell);
        allocator.collect_returned();

        assert_eq!(allocator.returning(), 1);
        assert_eq!(allocator.take(PageRetention::Evictable), None);

        give_back();
        allocator.collect_returned();

        assert_eq!(allocator.returning(), 0);
        assert_eq!(allocator.take(PageRetention::Evictable), Some(cell));
    }

    #[test]
    fn reserved_cells_go_only_to_the_pages_they_were_reserved_for() {
        let mut allocator = CellAllocator::new();
        allocator.add_chunk(chunk(1), 2);
        allocator.reserve(1);

        assert!(allocator.take(PageRetention::Evictable).is_some());
        assert_eq!(allocator.take(PageRetention::Evictable), None);
        assert!(allocator.take_reserved(PageRetention::Evictable).is_some());
    }

    #[test]
    fn a_draining_chunk_hands_out_no_cell_and_takes_none_back() {
        let mut allocator = CellAllocator::new();
        allocator.add_chunk(chunk(1), 2);
        let cell = allocator.take(PageRetention::Evictable).unwrap();
        allocator.drain_chunk(chunk(1));
        let give_back = allocator.give_back(cell);

        assert_eq!(allocator.returning(), 0);

        give_back();
        allocator.collect_returned();

        assert_eq!(allocator.available(), 0);
    }

    #[test]
    fn cells_of_a_released_chunk_never_return() {
        let mut allocator = CellAllocator::new();
        allocator.add_chunk(chunk(1), 1);
        let cell = allocator.take(PageRetention::Evictable).unwrap();
        let give_back = allocator.give_back(cell);
        allocator.remove_chunk(chunk(1));
        give_back();
        allocator.collect_returned();

        assert_eq!(allocator.take(PageRetention::Evictable), None);
    }
}
