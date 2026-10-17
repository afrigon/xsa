use crate::Buffer;
use crate::transfer_queue::STAGING_BLOCK_SIZE;

// One block of a staging ring, filled from its start until no submitted batch uses it.
pub(crate) struct StagingBlock {
    pub buffer: Buffer,
    pub used: usize,
    pub batches: u32,
}

impl StagingBlock {
    pub fn new(buffer: Buffer) -> StagingBlock {
        StagingBlock {
            buffer,
            used: 0,
            batches: 0,
        }
    }

    pub fn fit(&mut self, size: usize, alignment: usize) -> Option<usize> {
        let offset = self.used.next_multiple_of(alignment);

        if offset + size > STAGING_BLOCK_SIZE {
            return None;
        }

        self.used = offset + size;

        Some(offset)
    }
}
