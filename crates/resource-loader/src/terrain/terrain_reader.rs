use std::io::{Read, Seek, SeekFrom};

use anyhow::{Context, ensure};

use super::{CubeFace, LevelRecord, PageEncoding, PageEntry, PageKey, TerrainHeader};

/// Reads a `.terrain` file: its header and level index at once, page entries and page data on demand.
pub struct TerrainReader<R> {
    source: R,
    header: TerrainHeader,
    levels: Vec<LevelRecord>,
}

impl<R: Read + Seek> TerrainReader<R> {
    pub fn new(mut source: R) -> anyhow::Result<TerrainReader<R>> {
        let mut header = [0; TerrainHeader::SIZE];
        source.read_exact(&mut header).context("reading the header")?;
        let header = TerrainHeader::read(&header)?;
        let mut levels = Vec::new();

        for _ in 0..CubeFace::ALL.len() * header.level_count as usize {
            let mut record = [0; LevelRecord::SIZE];
            source.read_exact(&mut record).context("reading the level index")?;
            levels.push(LevelRecord::read(&record));
        }

        Ok(TerrainReader { source, header, levels })
    }

    pub fn header(&self) -> &TerrainHeader {
        &self.header
    }

    /// The present pages of a face at a level, sorted by key.
    pub fn entries(&mut self, face: CubeFace, level: u32) -> anyhow::Result<Vec<PageEntry>> {
        let record = self.record(face, level)?;
        let size = PageEntry::size(self.header.encoding);
        let mut bytes = vec![0; record.entry_count as usize * size];
        self.source.seek(SeekFrom::Start(record.entries_offset))?;
        self.source
            .read_exact(&mut bytes)
            .with_context(|| format!("reading the entries of face {face:?} at level {level}"))?;

        Ok(bytes
            .chunks_exact(size)
            .map(|entry| PageEntry::read(entry, self.header.encoding))
            .collect())
    }

    /// A page's entry, found by binary search over its face and level's entries, or `None` when it is absent.
    pub fn entry(&mut self, face: CubeFace, level: u32, key: PageKey) -> anyhow::Result<Option<PageEntry>> {
        let record = self.record(face, level)?;
        let size = PageEntry::size(self.header.encoding);
        let mut bytes = vec![0; size];
        let (mut low, mut high) = (0, record.entry_count);

        while low < high {
            let middle = low + (high - low) / 2;
            self.source
                .seek(SeekFrom::Start(record.entries_offset + middle * size as u64))?;
            self.source.read_exact(&mut bytes)?;
            let entry = PageEntry::read(&bytes, self.header.encoding);

            match entry.key.cmp(&key) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(Some(entry)),
            }
        }

        Ok(None)
    }

    /// A page's texels, decompressed: rows of texels top to bottom, or rows of blocks for compressed formats.
    pub fn page(&mut self, entry: &PageEntry) -> anyhow::Result<Vec<u8>> {
        ensure!(
            entry.range.is_some() == (self.header.encoding == PageEncoding::PageHeights),
            "page {:?} does not match the file's encoding",
            entry.key
        );

        let mut data = vec![0; entry.data_size as usize];
        self.source.seek(SeekFrom::Start(entry.data_offset))?;
        self.source
            .read_exact(&mut data)
            .with_context(|| format!("reading page {:?}", entry.key))?;
        let page = zstd::bulk::decompress(&data, self.header.page_size())
            .with_context(|| format!("decompressing page {:?}", entry.key))?;
        ensure!(
            page.len() == self.header.page_size(),
            "page {:?} holds {} bytes, expected {}",
            entry.key,
            page.len(),
            self.header.page_size()
        );

        Ok(page)
    }

    fn record(&self, face: CubeFace, level: u32) -> anyhow::Result<LevelRecord> {
        ensure!(
            level < self.header.level_count,
            "level {level}: the file has {} levels",
            self.header.level_count
        );

        Ok(self.levels[face.index() * self.header.level_count as usize + level as usize])
    }
}
