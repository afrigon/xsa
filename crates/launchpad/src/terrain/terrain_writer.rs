use std::io::Write;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use anyhow::Context;
use resource_loader::terrain::{CubeFace, HeightRange, LevelRecord, PageEntry, PageKey, TerrainHeader};

use super::{CubeMapInput, LayerEncoding, PageBuilder};

/// Writes a cube map as a `.terrain` file: every page of every level, encoded on every core.
pub struct TerrainWriter<'a> {
    input: &'a CubeMapInput,
    encoding: LayerEncoding,
}

struct EncodedPage {
    data: Vec<u8>,
    range: Option<HeightRange>,
}

impl<'a> TerrainWriter<'a> {
    // Decompression is as fast at any level, and pages are written once and read many times.
    const COMPRESSION_LEVEL: i32 = 19;

    pub fn new(input: &'a CubeMapInput, encoding: LayerEncoding) -> TerrainWriter<'a> {
        TerrainWriter { input, encoding }
    }

    pub fn write(&self, output: &mut impl Write) -> anyhow::Result<()> {
        let pages = self.encoded_pages()?;
        let header = TerrainHeader {
            format: self.encoding.page_format(),
            encoding: self.encoding.page_encoding(),
            level_count: self.input.level_count(),
        };
        let entry_size = PageEntry::size(header.encoding) as u64;
        let record_count = (CubeFace::ALL.len() * header.level_count as usize) as u64;
        let entries_start = TerrainHeader::SIZE as u64 + record_count * LevelRecord::SIZE as u64;
        let mut data_offset = entries_start + pages.len() as u64 * entry_size;
        let mut entries_offset = entries_start;
        header.write(output)?;

        for _ in CubeFace::ALL {
            for level in 0..header.level_count {
                let entry_count = TerrainWriter::level_page_count(level) as u64;
                LevelRecord {
                    entries_offset,
                    entry_count,
                }
                .write(output)?;
                entries_offset += entry_count * entry_size;
            }
        }

        for (key, page) in self.page_keys().map(|(_, _, key)| key).zip(&pages) {
            PageEntry {
                key,
                data_offset,
                data_size: page.data.len() as u32,
                range: page.range,
            }
            .write(output)?;
            data_offset += page.data.len() as u64;
        }

        for page in &pages {
            output.write_all(&page.data)?;
        }

        Ok(())
    }

    /// The number of pages the file holds.
    pub fn page_count(&self) -> usize {
        CubeFace::ALL.len()
            * (0..self.input.level_count())
                .map(TerrainWriter::level_page_count)
                .sum::<usize>()
    }

    fn level_page_count(level: u32) -> usize {
        1 << (2 * level)
    }

    // Every page in file order: by face, then level, then key. A level's pages cover its whole face, so its keys are
    // every Morton code below its page count.
    fn page_keys(&self) -> impl Iterator<Item = (CubeFace, u32, PageKey)> + '_ {
        CubeFace::ALL.into_iter().flat_map(move |face| {
            (0..self.input.level_count()).flat_map(move |level| {
                (0..TerrainWriter::level_page_count(level) as u64).map(move |key| (face, level, PageKey(key)))
            })
        })
    }

    fn encoded_pages(&self) -> anyhow::Result<Vec<EncodedPage>> {
        let builders: Vec<PageBuilder> = (0..self.input.level_count())
            .map(|level| PageBuilder::new(self.input, level, self.encoding))
            .collect();
        let pages: Vec<(CubeFace, u32, PageKey)> = self.page_keys().collect();
        let next = AtomicUsize::new(0);
        let workers = thread::available_parallelism().map_or(1, NonZero::get);
        let mut encoded: Vec<Option<EncodedPage>> = (0..pages.len()).map(|_| None).collect();

        let results = thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| -> anyhow::Result<Vec<(usize, EncodedPage)>> {
                        let mut done = Vec::new();

                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(&(face, level, key)) = pages.get(index) else {
                                return Ok(done);
                            };
                            let texels = builders[level as usize].page(face, key);
                            let (texels, range) = self.encoding.encode(&texels);
                            let data = zstd::bulk::compress(&texels, Self::COMPRESSION_LEVEL).with_context(|| {
                                format!("compressing page {key:?} of face {face:?} at level {level}")
                            })?;
                            done.push((index, EncodedPage { data, range }));
                        }
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|handle| handle.join().expect("a page encoder panicked"))
                .collect::<anyhow::Result<Vec<_>>>()
        })?;

        for (index, page) in results.into_iter().flatten() {
            encoded[index] = Some(page);
        }

        Ok(encoded.into_iter().map(Option::unwrap).collect())
    }
}
