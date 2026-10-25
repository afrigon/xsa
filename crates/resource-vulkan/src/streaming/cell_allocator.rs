use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::streaming::{CellAddress, ChunkId, PageRetention};

// The free cells of a pool's ready chunks. A cell given back returns only once the action it hands out has run:
// the streamer defers it until the GPU finished every frame that may still sample the cell and every copy that may
// still write it. Pinned pages take the lowest free cell and others the highest, so pinned pages gather in few
// chunks.
pub(crate) struct CellAllocator {
    chunks: BTreeMap<ChunkId, u32>,
    free: BTreeSet<CellAddress>,
    returned: Arc<Mutex<Vec<CellAddress>>>,
    returning: Arc<Mutex<usize>>,
}

impl CellAllocator {
    pub fn new() -> CellAllocator {
        CellAllocator {
            chunks: BTreeMap::new(),
            free: BTreeSet::new(),
            returned: Arc::new(Mutex::new(Vec::new())),
            returning: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_chunk(&mut self, chunk: ChunkId, cells: u32) {
        self.chunks.insert(chunk, cells);
        self.free.extend((0..cells).map(|cell| CellAddress { chunk, cell }));
    }

    pub fn remove_chunk(&mut self, chunk: ChunkId) {
        self.chunks.remove(&chunk);
        self.free.retain(|cell| cell.chunk != chunk);
    }

    pub fn take(&mut self, retention: PageRetention) -> Option<CellAddress> {
        match retention {
            PageRetention::Pinned => self.free.pop_first(),
            PageRetention::Evictable => self.free.pop_last(),
        }
    }

    // What returns `cell` once run.
    pub fn give_back(&self, cell: CellAddress) -> impl FnOnce() + Send + 'static {
        *CellAllocator::lock(&self.returning) += 1;
        let returned = Arc::clone(&self.returned);
        let returning = Arc::clone(&self.returning);

        move || {
            CellAllocator::lock(&returned).push(cell);
            *CellAllocator::lock(&returning) -= 1;
        }
    }

    // Frees the cells returned since the last call; those of released chunks are dropped.
    pub fn collect_returned(&mut self) {
        let returned = std::mem::take(&mut *CellAllocator::lock(&self.returned));
        let chunks = &self.chunks;
        self.free
            .extend(returned.into_iter().filter(|cell| chunks.contains_key(&cell.chunk)));
    }

    // Cells given back that have not returned yet.
    pub fn returning(&self) -> usize {
        *CellAllocator::lock(&self.returning)
    }

    pub fn capacity(&self) -> u64 {
        self.chunks.values().map(|&cells| cells as u64).sum()
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
