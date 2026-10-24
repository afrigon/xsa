use memory_vulkan::BindlessTable;

use crate::LoadEnvironment;
use crate::streaming::{
    CellAddress, CellAllocator, Chunk, ChunkGrid, ChunkId, PageFormat, PageOwner, PageRetention, PoolCapacity,
};

const CHUNK_TARGET_SIZE: u64 = 32 * 1024 * 1024;
// The baseline GPU's maxImageDimension2D.
const MAX_CHUNK_EXTENT: u32 = 32768;

// The chunks holding every page of one format, across sets. Each chunk is about the same size whatever the format,
// so pools grow and shrink in equal steps.
pub(crate) struct PagePool {
    pub format: PageFormat,
    chunks: Vec<Chunk>,
    pub allocator: CellAllocator,
    // Pages left in draining chunks that have not started moving out, each with a cell reserved for it.
    pub survivors: Vec<PageOwner>,
}

impl PagePool {
    pub fn new(format: PageFormat) -> PagePool {
        PagePool {
            format,
            chunks: Vec::new(),
            allocator: CellAllocator::new(),
            survivors: Vec::new(),
        }
    }

    // A grid as close to square as the target size allows.
    pub fn grid(&self) -> ChunkGrid {
        let extent = self.format.extent;
        let target = (CHUNK_TARGET_SIZE / self.format.page_size() as u64).max(1) as u32;
        let columns = target.isqrt().clamp(1, MAX_CHUNK_EXTENT / extent.width);
        let rows = (target / columns).clamp(1, MAX_CHUNK_EXTENT / extent.height);

        ChunkGrid {
            columns,
            rows,
            page_extent: extent,
        }
    }

    // The video memory one more chunk takes: what the driver allocated for the last one, or the raw pages before
    // the first.
    pub fn chunk_size(&self) -> u64 {
        self.chunks.last().map_or_else(
            || self.grid().cells() as u64 * self.format.page_size() as u64,
            |chunk| chunk.size,
        )
    }

    pub fn add_chunk(&mut self, environment: &LoadEnvironment, id: ChunkId) -> anyhow::Result<()> {
        let chunk = Chunk::new(environment, &self.format, self.grid(), id)?;
        tracing::debug!(
            "a chunk of {} {:?} pages takes {} bytes for {} bytes of pages",
            chunk.grid.cells(),
            self.format.format,
            chunk.size,
            chunk.grid.cells() as u64 * self.format.page_size() as u64,
        );
        self.chunks.push(chunk);

        Ok(())
    }

    // Frees the cells of chunks that became ready, and the cells returned since the last update.
    pub fn update(&mut self, bindless: &BindlessTable) {
        for chunk in &mut self.chunks {
            if chunk.poll(bindless) {
                self.allocator.add_chunk(chunk.id, chunk.grid.cells());
            }
        }

        self.allocator.collect_returned();
    }

    pub fn take_cell(&mut self, retention: PageRetention) -> Option<CellAddress> {
        self.allocator.take(retention)
    }

    pub fn chunk(&self, id: ChunkId) -> Option<&Chunk> {
        self.chunks.iter().find(|chunk| chunk.id == id)
    }

    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    // Takes the chunk out of the pool; its pages must be evicted first. Dropping it frees it through the deletion
    // queue.
    pub fn remove_chunk(&mut self, id: ChunkId) -> Option<Chunk> {
        let index = self.chunks.iter().position(|chunk| chunk.id == id)?;
        self.allocator.remove_chunk(id);

        Some(self.chunks.remove(index))
    }

    // The pool as eviction sees it, unless it is already shrinking or holds no ready chunk.
    pub fn capacity(&self) -> Option<PoolCapacity> {
        let ready = self.chunks.iter().filter(|chunk| chunk.ready().is_some()).count();

        if ready == 0 || self.is_draining() {
            return None;
        }

        Some(PoolCapacity {
            format: self.format,
            cells_per_chunk: self.grid().cells() as usize,
            chunk_size: self.chunk_size(),
            free_cells: self.allocator.available() + self.allocator.returning(),
        })
    }

    // The video memory one cell takes.
    pub fn cell_size(&self) -> u64 {
        self.chunk_size() / self.grid().cells() as u64
    }

    pub fn is_draining(&self) -> bool {
        self.allocator.draining().next().is_some()
    }

    pub fn is_growing(&self) -> bool {
        self.chunks.iter().any(Chunk::is_preparing)
    }

    pub fn video_bytes(&self) -> u64 {
        self.chunks.iter().map(|chunk| chunk.size).sum()
    }
}
