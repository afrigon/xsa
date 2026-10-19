use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use crate::Job;
use crate::streaming::{
    ByteUse, CancelledUpload, CellAddress, CellUse, ChunkId, MissingPage, PageFormat, PageId, ReadOutcome, ReadPage,
    ReadResult, Residency, SetId, SetStatistics,
};

// A streamed set of any page source, as the streamer drives it.
pub(crate) trait AnySet: Send + Sync {
    fn id(&self) -> SetId;

    fn name(&self) -> &str;

    fn format(&self) -> PageFormat;

    fn residency(&self) -> Residency;

    // Makes the latest requests the wanted pages; returns the uploads of pages no longer wanted.
    fn apply(&self, now: u64) -> Vec<CancelledUpload>;

    fn missing(&self) -> Vec<MissingPage>;

    // Marks `page` as reading and returns the job that reads it into `finished`, counted in `reading` until it ends.
    fn read_job(&self, page: PageId, finished: &Arc<Mutex<Vec<ReadPage>>>, reading: &Arc<AtomicUsize>) -> Option<Job>;

    fn finish_read(&self, page: PageId, result: ReadResult) -> ReadOutcome;

    fn start_upload(&self, page: PageId, cell: CellAddress) -> Option<Arc<[u8]>>;

    fn finish_upload(&self, page: PageId, cell: CellAddress, slot: u32, offset: ash::vk::Offset2D);

    fn evict_cell(&self, page: PageId) -> Option<CellAddress>;

    fn evict_chunk(&self, chunk: ChunkId) -> Vec<CancelledUpload>;

    fn evict_bytes(&self, page: PageId);

    // Empties the set; returns the cells its pages held.
    fn release(&self) -> Vec<CellAddress>;

    fn cell_uses(&self) -> Vec<CellUse>;

    fn byte_uses(&self) -> Vec<ByteUse>;

    fn ram_bytes(&self) -> u64;

    fn statistics(&self) -> SetStatistics;
}
