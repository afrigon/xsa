use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;

use crate::streaming::{
    ByteUse, CancelledUpload, CellAddress, CellUse, ChunkId, MissingPage, PageId, PageNeed, PagePlacement, PageRecord,
    PageRequest, PageRetention, PageState, ReadOutcome, ReadResult, Residency, SetStatistics,
};

// One streamed set's pages: which are wanted, what each holds, and the latest requests and pins. Pages nobody wants
// and nothing holds are forgotten; pages the source has no data for are remembered, so they are never read again.
pub(crate) struct SetTable<K> {
    ids: HashMap<K, PageId>,
    records: HashMap<PageId, PageRecord<K>>,
    next_page: u64,
    requests: Vec<PageRequest<K>>,
    pins: HashSet<K>,
    wanted: Vec<PageId>,
    ram_bytes: u64,
    loads: u64,
    evictions: u64,
    pub residency: Residency,
}

impl<K: Copy + Eq + Hash> SetTable<K> {
    pub fn new() -> SetTable<K> {
        SetTable {
            ids: HashMap::new(),
            records: HashMap::new(),
            next_page: 0,
            requests: Vec::new(),
            pins: HashSet::new(),
            wanted: Vec::new(),
            ram_bytes: 0,
            loads: 0,
            evictions: 0,
            residency: Residency::Live,
        }
    }

    pub fn request(&mut self, requests: impl IntoIterator<Item = PageRequest<K>>) {
        self.requests.clear();
        self.requests.extend(requests);
    }

    pub fn pin(&mut self, key: K) {
        self.pins.insert(key);
    }

    pub fn unpin(&mut self, key: K) {
        self.pins.remove(&key);
    }

    // Makes the latest requests and the pins the wanted pages, used in update `now`. Pages no longer wanted stop
    // reading and lose their uploads, whose cells the pool takes back.
    pub fn apply(&mut self, now: u64) -> Vec<CancelledUpload> {
        let previous = std::mem::take(&mut self.wanted);

        for page in &previous {
            if let Some(record) = self.records.get_mut(page) {
                record.score = None;
            }
        }

        let requests = std::mem::take(&mut self.requests);
        let pins: Vec<_> = self
            .pins
            .iter()
            .map(|&key| PageRequest {
                key,
                score: f32::INFINITY,
            })
            .collect();

        for request in requests.iter().copied().chain(pins) {
            let page = self.intern(request.key);
            let record = self.records.get_mut(&page).expect("an interned page has a record");
            record.last_use = now;

            if let PageState::Reading { cancel } = &record.state {
                cancel.store(false, Ordering::Release);
            }

            match record.score {
                None => {
                    record.score = Some(request.score);
                    self.wanted.push(page);
                }
                Some(score) => record.score = Some(score.max(request.score)),
            }
        }

        self.requests = requests;
        let mut cancelled = Vec::new();

        for page in previous {
            let Some(record) = self.records.get_mut(&page) else {
                continue;
            };

            if record.score.is_some() {
                continue;
            }

            match record.state {
                PageState::Reading { ref cancel } => cancel.store(true, Ordering::Release),
                PageState::Uploading { cell } => {
                    record.state = PageState::Cached;
                    cancelled.push(CancelledUpload { page, cell });
                }
                _ => {}
            }

            self.forget_if_empty(page);
        }

        cancelled
    }

    // The wanted pages without a cell, and what each needs next.
    pub fn missing(&self) -> Vec<MissingPage> {
        self.wanted
            .iter()
            .filter_map(|&page| {
                let record = &self.records[&page];
                let need = match record.state {
                    PageState::Unloaded => PageNeed::Read,
                    PageState::Cached => PageNeed::Cell,
                    _ => return None,
                };

                Some(MissingPage {
                    page,
                    score: record.score.expect("a wanted page has a score"),
                    need,
                    retention: self.retention(record.key),
                })
            })
            .collect()
    }

    // Marks `page` as reading; returns its key and the flag that cancels the read.
    pub fn start_read(&mut self, page: PageId) -> Option<(K, Arc<AtomicBool>)> {
        let record = self.records.get_mut(&page)?;

        if !matches!(record.state, PageState::Unloaded) {
            return None;
        }

        let cancel = Arc::new(AtomicBool::new(false));
        record.state = PageState::Reading {
            cancel: Arc::clone(&cancel),
        };

        Some((record.key, cancel))
    }

    // A read job reported back. Bytes read after the page stopped being wanted still join the RAM cache.
    pub fn finish_read(&mut self, page: PageId, result: ReadResult) -> ReadOutcome {
        let Some(record) = self.records.get_mut(&page) else {
            return ReadOutcome::Ignored;
        };

        if !matches!(record.state, PageState::Reading { .. }) {
            return ReadOutcome::Ignored;
        }

        match result {
            ReadResult::Cancelled => {
                record.state = PageState::Unloaded;
                self.forget_if_empty(page);
                ReadOutcome::Cancelled
            }
            ReadResult::Bytes(bytes) => {
                self.ram_bytes += bytes.len() as u64;
                self.loads += 1;
                record.bytes = Some(bytes);
                record.state = PageState::Cached;
                ReadOutcome::Cached
            }
            ReadResult::Absent => {
                record.state = PageState::Absent;
                ReadOutcome::Absent
            }
            ReadResult::Failed(err) => {
                record.state = PageState::Failed;
                ReadOutcome::Failed(err)
            }
        }
    }

    // Starts copying a cached page into `cell`; returns its bytes.
    pub fn start_upload(&mut self, page: PageId, cell: CellAddress) -> Option<Arc<[u8]>> {
        let record = self.records.get_mut(&page)?;

        if !matches!(record.state, PageState::Cached) {
            return None;
        }

        record.state = PageState::Uploading { cell };
        record.bytes.clone()
    }

    pub fn finish_upload(&mut self, page: PageId, cell: CellAddress, slot: u32, offset: vk::Offset2D) {
        if let Some(record) = self.records.get_mut(&page)
            && record.state.cell() == Some(cell)
        {
            record.state = PageState::Resident { cell, slot, offset };
        }
    }

    // Takes `page` out of its cell; it stays cached while its bytes are in RAM. Returns the cell it held.
    pub fn evict_cell(&mut self, page: PageId) -> Option<CellAddress> {
        let record = self.records.get_mut(&page)?;
        let cell = record.state.cell()?;
        record.state = match record.bytes {
            Some(_) => PageState::Cached,
            None => PageState::Unloaded,
        };
        self.evictions += 1;
        self.forget_if_empty(page);

        Some(cell)
    }

    // Takes every page out of `chunk`, which is being released.
    pub fn evict_chunk(&mut self, chunk: ChunkId) -> Vec<CancelledUpload> {
        let pages: Vec<_> = self
            .records
            .iter()
            .filter(|(_, record)| record.state.cell().is_some_and(|cell| cell.chunk == chunk))
            .map(|(&page, _)| page)
            .collect();

        pages
            .into_iter()
            .filter_map(|page| {
                Some(CancelledUpload {
                    page,
                    cell: self.evict_cell(page)?,
                })
            })
            .collect()
    }

    // Drops `page`'s bytes from RAM; a page in a cell keeps it.
    pub fn evict_bytes(&mut self, page: PageId) {
        let Some(record) = self.records.get_mut(&page) else {
            return;
        };

        if let Some(bytes) = record.bytes.take() {
            self.ram_bytes -= bytes.len() as u64;
        }

        if matches!(record.state, PageState::Cached) {
            record.state = PageState::Unloaded;
        }

        self.forget_if_empty(page);
    }

    // Cancels every read and empties the table; returns the cells its pages held.
    pub fn release(&mut self) -> Vec<CellAddress> {
        let cells = self
            .records
            .values()
            .filter_map(|record| {
                if let PageState::Reading { cancel } = &record.state {
                    cancel.store(true, Ordering::Release);
                }

                record.state.cell()
            })
            .collect();
        *self = SetTable::new();

        cells
    }

    // Where shaders read `key` from: the page itself or its nearest resident coarser page.
    pub fn placement(&self, key: K, coarser: impl Fn(K) -> Option<K>) -> Option<PagePlacement<K>> {
        let mut current = Some(key);

        while let Some(key) = current {
            if let Some(record) = self.ids.get(&key).and_then(|page| self.records.get(page))
                && let PageState::Resident { slot, offset, .. } = record.state
            {
                return Some(PagePlacement {
                    page: key,
                    slot,
                    offset,
                });
            }

            current = coarser(key);
        }

        None
    }

    pub fn bytes(&self, key: K, coarser: impl Fn(K) -> Option<K>) -> Option<(K, Arc<[u8]>)> {
        let mut current = Some(key);

        while let Some(key) = current {
            if let Some(bytes) = self
                .ids
                .get(&key)
                .and_then(|page| self.records.get(page))
                .and_then(|record| record.bytes.as_ref())
            {
                return Some((key, Arc::clone(bytes)));
            }

            current = coarser(key);
        }

        None
    }

    pub fn cell_uses(&self) -> impl Iterator<Item = CellUse> + '_ {
        self.records.iter().filter_map(|(&page, record)| {
            Some(CellUse {
                page,
                cell: record.state.cell()?,
                last_use: record.last_use,
                score: record.score,
                retention: self.retention(record.key),
            })
        })
    }

    pub fn byte_uses(&self) -> impl Iterator<Item = ByteUse> + '_ {
        self.records.iter().filter_map(|(&page, record)| {
            Some(ByteUse {
                page,
                bytes: record.bytes.as_ref()?.len() as u64,
                last_use: record.last_use,
                score: record.score,
            })
        })
    }

    pub fn ram_bytes(&self) -> u64 {
        self.ram_bytes
    }

    pub fn statistics(&self) -> SetStatistics {
        SetStatistics {
            resident: self
                .records
                .values()
                .filter(|record| matches!(record.state, PageState::Resident { .. }))
                .count(),
            pending: self.missing().len(),
            ram_bytes: self.ram_bytes,
            loads: self.loads,
            evictions: self.evictions,
        }
    }

    fn retention(&self, key: K) -> PageRetention {
        if self.pins.contains(&key) {
            PageRetention::Pinned
        } else {
            PageRetention::Evictable
        }
    }

    fn intern(&mut self, key: K) -> PageId {
        if let Some(&page) = self.ids.get(&key) {
            return page;
        }

        self.next_page += 1;
        let page = PageId { value: self.next_page };
        self.ids.insert(key, page);
        self.records.insert(
            page,
            PageRecord {
                key,
                state: PageState::Unloaded,
                bytes: None,
                last_use: 0,
                score: None,
            },
        );

        page
    }

    fn forget_if_empty(&mut self, page: PageId) {
        let Some(record) = self.records.get(&page) else {
            return;
        };

        if record.score.is_none() && record.bytes.is_none() && matches!(record.state, PageState::Unloaded) {
            self.ids.remove(&record.key);
            self.records.remove(&page);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A quadtree page: its four children at the next level are `index * 4` to `index * 4 + 3`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Key {
        level: u8,
        index: u32,
    }

    fn key(level: u8, index: u32) -> Key {
        Key { level, index }
    }

    fn coarser(page: Key) -> Option<Key> {
        (page.level > 0).then(|| key(page.level - 1, page.index / 4))
    }

    fn wanted(key: Key, score: f32) -> PageRequest<Key> {
        PageRequest { key, score }
    }

    fn bytes() -> ReadResult {
        ReadResult::Bytes(Arc::from(vec![0u8; 16]))
    }

    fn cell(cell: u32) -> CellAddress {
        CellAddress {
            chunk: ChunkId { value: 1 },
            cell,
        }
    }

    fn page_of(table: &SetTable<Key>, key: Key) -> PageId {
        table.ids[&key]
    }

    // Reads and uploads `key` into `cell`, as the streamer would.
    fn make_resident(table: &mut SetTable<Key>, key: Key, into: CellAddress) {
        let page = page_of(table, key);
        table.start_read(page).unwrap();
        table.finish_read(page, bytes());
        table.start_upload(page, into).unwrap();
        table.finish_upload(page, into, 7, vk::Offset2D { x: 136, y: 0 });
    }

    #[test]
    fn wanted_pages_are_missing_until_read_then_until_placed() {
        let mut table = SetTable::new();
        table.request([wanted(key(0, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(0, 0));

        assert_eq!(table.missing()[0].need, PageNeed::Read);

        table.start_read(page).unwrap();

        assert!(table.missing().is_empty());

        table.finish_read(page, bytes());

        assert_eq!(table.missing()[0].need, PageNeed::Cell);
    }

    #[test]
    fn a_missing_page_falls_back_to_its_nearest_resident_coarser_page() {
        let mut table = SetTable::new();
        table.request([wanted(key(0, 0), 1.0), wanted(key(2, 9), 4.0)]);
        table.apply(1);
        make_resident(&mut table, key(0, 0), cell(3));

        assert_eq!(
            table.placement(key(2, 9), coarser),
            Some(PagePlacement {
                page: key(0, 0),
                slot: 7,
                offset: vk::Offset2D { x: 136, y: 0 }
            })
        );
    }

    #[test]
    fn nothing_resident_has_no_placement() {
        let table = SetTable::<Key>::new();

        assert_eq!(table.placement(key(1, 2), coarser), None);
    }

    #[test]
    fn a_page_without_bytes_reads_its_nearest_coarser_page_in_ram() {
        let mut table = SetTable::new();
        table.request([wanted(key(0, 0), 1.0), wanted(key(2, 9), 4.0)]);
        table.apply(1);
        let page = page_of(&table, key(0, 0));
        table.start_read(page).unwrap();
        table.finish_read(page, bytes());

        let (found, _) = table.bytes(key(2, 9), coarser).unwrap();
        assert_eq!(found, key(0, 0));
        assert!(SetTable::<Key>::new().bytes(key(2, 9), coarser).is_none());
    }

    #[test]
    fn an_unwanted_read_is_cancelled_and_an_unwanted_upload_gives_its_cell_back() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0), wanted(key(1, 1), 1.0)]);
        table.apply(1);
        let (_, cancel) = table.start_read(page_of(&table, key(1, 0))).unwrap();
        let uploading = page_of(&table, key(1, 1));
        table.start_read(uploading).unwrap();
        table.finish_read(uploading, bytes());
        table.start_upload(uploading, cell(5)).unwrap();
        table.request([]);
        let cancelled = table.apply(2);

        assert!(cancel.load(Ordering::Acquire));
        assert_eq!(cancelled.len(), 1);
        assert_eq!(cancelled[0].cell, cell(5));
    }

    #[test]
    fn a_cancelled_read_that_finished_keeps_its_bytes_in_ram() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(1, 0));
        table.start_read(page).unwrap();
        table.request([]);
        table.apply(2);
        table.finish_read(page, bytes());

        assert_eq!(table.ram_bytes(), 16);
    }

    #[test]
    fn a_page_wanted_again_while_its_read_is_cancelled_keeps_reading() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(1, 0));
        let (_, cancel) = table.start_read(page).unwrap();
        table.request([]);
        table.apply(2);
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(3);

        assert!(!cancel.load(Ordering::Acquire));
        assert!(table.missing().is_empty());
    }

    #[test]
    fn a_read_skipped_after_cancelling_is_read_again_when_wanted() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(1, 0));
        table.start_read(page).unwrap();
        table.finish_read(page, ReadResult::Cancelled);

        assert_eq!(table.missing()[0].need, PageNeed::Read);
    }

    #[test]
    fn evicted_pages_stay_cached_in_ram() {
        let mut table = SetTable::new();
        table.request([wanted(key(0, 0), 1.0)]);
        table.apply(1);
        make_resident(&mut table, key(0, 0), cell(0));
        let page = page_of(&table, key(0, 0));

        assert_eq!(table.evict_cell(page), Some(cell(0)));
        assert_eq!(table.missing()[0].need, PageNeed::Cell);
    }

    #[test]
    fn an_absent_page_is_never_read_again() {
        let mut table = SetTable::new();
        table.request([wanted(key(3, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(3, 0));
        table.start_read(page).unwrap();
        table.finish_read(page, ReadResult::Absent);
        table.request([wanted(key(3, 0), 1.0)]);
        table.apply(2);

        assert!(table.missing().is_empty());
    }

    #[test]
    fn pins_are_wanted_above_every_request_and_never_evictable() {
        let mut table = SetTable::new();
        table.pin(key(0, 0));
        table.request([wanted(key(1, 0), 100.0)]);
        table.apply(1);
        let missing = table.missing();
        let pinned = missing
            .iter()
            .find(|page| page.retention == PageRetention::Pinned)
            .unwrap();

        assert_eq!(pinned.score, f32::INFINITY);
    }

    #[test]
    fn unwanted_pages_without_bytes_are_forgotten() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(1);
        table.request([]);
        table.apply(2);

        assert!(table.records.is_empty());
    }

    #[test]
    fn evicting_bytes_of_a_cached_unwanted_page_forgets_it() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0)]);
        table.apply(1);
        let page = page_of(&table, key(1, 0));
        table.start_read(page).unwrap();
        table.finish_read(page, bytes());
        table.request([]);
        table.apply(2);
        table.evict_bytes(page);

        assert_eq!(table.ram_bytes(), 0);
        assert!(table.records.is_empty());
    }

    #[test]
    fn releasing_a_chunk_evicts_only_its_pages() {
        let mut table = SetTable::new();
        table.request([wanted(key(1, 0), 1.0), wanted(key(1, 1), 1.0)]);
        table.apply(1);
        let other = CellAddress {
            chunk: ChunkId { value: 2 },
            cell: 0,
        };
        make_resident(&mut table, key(1, 0), cell(0));
        make_resident(&mut table, key(1, 1), other);

        assert_eq!(table.evict_chunk(ChunkId { value: 1 }).len(), 1);
        assert_eq!(
            table.placement(key(1, 1), coarser).map(|placement| placement.page),
            Some(key(1, 1))
        );
    }
}
