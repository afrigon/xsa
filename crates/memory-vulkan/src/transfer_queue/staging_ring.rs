use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::transfer_queue::{StagingBlock, StagingSlice};
use crate::{Buffer, MemoryContext, MemoryError, QueueSharing};

/// The size of each block of a staging ring: one 4096² BC7 level, or about a thousand 128² terrain pages.
pub(crate) const STAGING_BLOCK_SIZE: usize = 16 * 1024 * 1024;

// Host-visible memory copies pass through, grown a block at a time while copies wait for room. A block is reused once
// no submitted batch uses it; the blocks the batch being built writes are open until it is submitted or abandoned.
pub(crate) struct StagingRing {
    memory: MemoryContext,
    name: &'static str,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
    blocks: Vec<StagingBlock>,
    current: Option<usize>,
    open: Vec<usize>,
}

impl StagingRing {
    pub fn new(
        memory: &MemoryContext,
        name: &'static str,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> StagingRing {
        StagingRing {
            memory: memory.clone(),
            name,
            usage,
            location,
            blocks: Vec::new(),
            current: None,
            open: Vec::new(),
        }
    }

    pub fn allocate(&mut self, size: usize, alignment: usize) -> Result<StagingSlice, MemoryError> {
        if let Some(current) = self.current {
            if self.is_free(current) {
                self.blocks[current].used = 0;
            }

            if let Some(offset) = self.blocks[current].fit(size, alignment) {
                self.open(current);
                return Ok(StagingSlice { block: current, offset });
            }
        }

        let index = match (0..self.blocks.len()).find(|&index| Some(index) != self.current && self.is_free(index)) {
            Some(index) => {
                self.blocks[index].used = 0;
                index
            }
            None => self.grow()?,
        };
        self.current = Some(index);
        self.open(index);
        let offset = self.blocks[index]
            .fit(size, alignment)
            .expect("a piece's copy fits an empty block");

        Ok(StagingSlice { block: index, offset })
    }

    pub fn buffer(&self, block: usize) -> &Buffer {
        &self.blocks[block].buffer
    }

    pub fn buffer_mut(&mut self, block: usize) -> &mut Buffer {
        &mut self.blocks[block].buffer
    }

    // The open blocks now belong to a submitted batch, which releases them once it completes.
    pub fn seal(&mut self) -> Vec<usize> {
        for &block in &self.open {
            self.blocks[block].batches += 1;
        }

        std::mem::take(&mut self.open)
    }

    pub fn abandon(&mut self) {
        self.open.clear();
    }

    pub fn release(&mut self, blocks: &[usize]) {
        for &block in blocks {
            self.blocks[block].batches -= 1;
        }
    }

    // Keeps one block once nothing waits for staging; the others are freed through the deletion queue.
    pub fn shrink(&mut self) {
        debug_assert!(self.open.is_empty() && self.blocks.iter().all(|block| block.batches == 0));
        self.blocks.truncate(1);
        self.current = None;
    }

    fn is_free(&self, block: usize) -> bool {
        self.blocks[block].batches == 0 && !self.open.contains(&block)
    }

    fn open(&mut self, block: usize) {
        if !self.open.contains(&block) {
            self.open.push(block);
        }
    }

    fn grow(&mut self) -> Result<usize, MemoryError> {
        let buffer = Buffer::new(
            &self.memory,
            self.name,
            STAGING_BLOCK_SIZE as u64,
            self.usage,
            self.location,
            &QueueSharing::Exclusive,
        )?;
        self.blocks.push(StagingBlock::new(buffer));

        Ok(self.blocks.len() - 1)
    }
}
