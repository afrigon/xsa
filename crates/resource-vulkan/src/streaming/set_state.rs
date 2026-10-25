use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::ensure;

use crate::streaming::{
    AnySet, ByteUse, CancelledUpload, CellAddress, CellUse, ChunkId, MissingPage, PageFormat, PageId, PageOwner,
    PageSource, ReadOutcome, ReadPage, ReadResult, Residency, SetId, SetStatistics, SetTable,
};
use crate::{Job, JobOwner};

// One streamed set: its page source and its table, shared by its handle and the streamer.
pub(crate) struct SetState<S: PageSource> {
    pub id: SetId,
    name: String,
    format: PageFormat,
    source: Arc<S>,
    table: Mutex<SetTable<S::Key>>,
}

impl<S: PageSource> SetState<S> {
    pub fn new(id: SetId, name: String, format: PageFormat, source: S) -> SetState<S> {
        SetState {
            id,
            name,
            format,
            source: Arc::new(source),
            table: Mutex::new(SetTable::new()),
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn table(&self) -> MutexGuard<'_, SetTable<S::Key>> {
        self.table.lock().expect("a streamed set's lock is poisoned")
    }
}

impl<S: PageSource> AnySet for SetState<S> {
    fn id(&self) -> SetId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> PageFormat {
        self.format
    }

    fn residency(&self) -> Residency {
        self.table().residency
    }

    fn apply(&self, now: u64) -> Vec<CancelledUpload> {
        self.table().apply(now)
    }

    fn missing(&self) -> Vec<MissingPage> {
        self.table().missing()
    }

    fn read_job(&self, page: PageId, finished: &Arc<Mutex<Vec<ReadPage>>>, reading: &Arc<AtomicUsize>) -> Option<Job> {
        let (key, cancel) = self.table().start_read(page)?;
        let source = Arc::clone(&self.source);
        let finished = Arc::clone(finished);
        let reading = Arc::clone(reading);
        let owner = PageOwner { set: self.id, page };
        let page_size = self.format.page_size();
        reading.fetch_add(1, Ordering::AcqRel);

        Some(Job {
            owner: JobOwner::Page,
            run: Box::new(move || {
                let result = if cancel.load(Ordering::Acquire) {
                    ReadResult::Cancelled
                } else {
                    let checked = source.read(key).and_then(|bytes| {
                        if let Some(bytes) = &bytes {
                            ensure!(
                                bytes.len() == page_size,
                                "page {key:?} has {} bytes, its format holds {page_size}",
                                bytes.len()
                            );
                        }

                        Ok(bytes)
                    });

                    match checked {
                        Ok(Some(bytes)) => ReadResult::Bytes(Arc::from(bytes)),
                        Ok(None) => ReadResult::Absent,
                        Err(err) => ReadResult::Failed(err),
                    }
                };
                finished
                    .lock()
                    .expect("the finished reads' lock is poisoned")
                    .push(ReadPage { owner, result });
                reading.fetch_sub(1, Ordering::AcqRel);
            }),
        })
    }

    fn finish_read(&self, page: PageId, result: ReadResult) -> ReadOutcome {
        self.table().finish_read(page, result)
    }

    fn start_upload(&self, page: PageId, cell: CellAddress) -> Option<Arc<[u8]>> {
        self.table().start_upload(page, cell)
    }

    fn finish_upload(&self, page: PageId, cell: CellAddress, slot: u32, offset: ash::vk::Offset2D) {
        self.table().finish_upload(page, cell, slot, offset);
    }

    fn evict_cell(&self, page: PageId) -> Option<CellAddress> {
        self.table().evict_cell(page)
    }

    fn evict_chunk(&self, chunk: ChunkId) -> Vec<CancelledUpload> {
        self.table().evict_chunk(chunk)
    }

    fn evict_bytes(&self, page: PageId) {
        self.table().evict_bytes(page);
    }

    fn release(&self) -> Vec<CellAddress> {
        self.table().release()
    }

    fn cell_uses(&self) -> Vec<CellUse> {
        self.table().cell_uses().collect()
    }

    fn byte_uses(&self) -> Vec<ByteUse> {
        self.table().byte_uses().collect()
    }

    fn ram_bytes(&self) -> u64 {
        self.table().ram_bytes()
    }

    fn statistics(&self) -> SetStatistics {
        self.table().statistics()
    }
}
