use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::streaming::{ChunkId, LayerAddress, PageRetention};

// The free layers of a pool's ready chunks. A layer given back returns only once the action it hands out has run:
// the streamer defers it until the GPU finished every frame that may still sample the layer and every copy that may
// still write it. Pinned pages take the lowest free layer and others the highest, so pinned pages gather in few
// chunks.
pub(crate) struct LayerAllocator {
    chunks: BTreeMap<ChunkId, u32>,
    free: BTreeSet<LayerAddress>,
    returned: Arc<Mutex<Vec<LayerAddress>>>,
    returning: Arc<Mutex<usize>>,
}

impl LayerAllocator {
    pub fn new() -> LayerAllocator {
        LayerAllocator {
            chunks: BTreeMap::new(),
            free: BTreeSet::new(),
            returned: Arc::new(Mutex::new(Vec::new())),
            returning: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_chunk(&mut self, chunk: ChunkId, layers: u32) {
        self.chunks.insert(chunk, layers);
        self.free.extend((0..layers).map(|layer| LayerAddress { chunk, layer }));
    }

    pub fn remove_chunk(&mut self, chunk: ChunkId) {
        self.chunks.remove(&chunk);
        self.free.retain(|layer| layer.chunk != chunk);
    }

    pub fn take(&mut self, retention: PageRetention) -> Option<LayerAddress> {
        match retention {
            PageRetention::Pinned => self.free.pop_first(),
            PageRetention::Evictable => self.free.pop_last(),
        }
    }

    // What returns `layer` once run.
    pub fn give_back(&self, layer: LayerAddress) -> impl FnOnce() + Send + 'static {
        *LayerAllocator::lock(&self.returning) += 1;
        let returned = Arc::clone(&self.returned);
        let returning = Arc::clone(&self.returning);

        move || {
            LayerAllocator::lock(&returned).push(layer);
            *LayerAllocator::lock(&returning) -= 1;
        }
    }

    // Frees the layers returned since the last call; those of released chunks are dropped.
    pub fn collect_returned(&mut self) {
        let returned = std::mem::take(&mut *LayerAllocator::lock(&self.returned));
        let chunks = &self.chunks;
        self.free
            .extend(returned.into_iter().filter(|layer| chunks.contains_key(&layer.chunk)));
    }

    // Layers given back that have not returned yet.
    pub fn returning(&self) -> usize {
        *LayerAllocator::lock(&self.returning)
    }

    pub fn capacity(&self) -> u64 {
        self.chunks.values().map(|&layers| layers as u64).sum()
    }

    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().expect("a layer allocator's lock is poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(value: u64) -> ChunkId {
        ChunkId { value }
    }

    #[test]
    fn pinned_pages_take_the_lowest_layers_and_others_the_highest() {
        let mut allocator = LayerAllocator::new();
        allocator.add_chunk(chunk(1), 2);
        allocator.add_chunk(chunk(2), 2);

        assert_eq!(
            allocator.take(PageRetention::Pinned),
            Some(LayerAddress {
                chunk: chunk(1),
                layer: 0
            })
        );
        assert_eq!(
            allocator.take(PageRetention::Evictable),
            Some(LayerAddress {
                chunk: chunk(2),
                layer: 1
            })
        );
    }

    #[test]
    fn a_layer_given_back_is_free_only_once_its_return_runs() {
        let mut allocator = LayerAllocator::new();
        allocator.add_chunk(chunk(1), 1);
        let layer = allocator.take(PageRetention::Evictable).unwrap();
        let give_back = allocator.give_back(layer);
        allocator.collect_returned();

        assert_eq!(allocator.returning(), 1);
        assert_eq!(allocator.take(PageRetention::Evictable), None);

        give_back();
        allocator.collect_returned();

        assert_eq!(allocator.returning(), 0);
        assert_eq!(allocator.take(PageRetention::Evictable), Some(layer));
    }

    #[test]
    fn layers_of_a_released_chunk_never_return() {
        let mut allocator = LayerAllocator::new();
        allocator.add_chunk(chunk(1), 1);
        let layer = allocator.take(PageRetention::Evictable).unwrap();
        let give_back = allocator.give_back(layer);
        allocator.remove_chunk(chunk(1));
        give_back();
        allocator.collect_returned();

        assert_eq!(allocator.take(PageRetention::Evictable), None);
    }
}
