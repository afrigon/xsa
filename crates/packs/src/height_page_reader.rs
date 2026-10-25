use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, ensure};
use resource_loader::terrain::{PageAddress, PageEncoding, TerrainReader};
use resource_loader::texture::TextureFormat;
use xsa_core::terrain::HeightPage;

// A body's height pages in `data/terrains/<body>.terrain`. Each read opens the file anew, so readers on several
// threads never share a position in it, and creating one reads nothing.
pub struct HeightPageReader {
    path: PathBuf,
}

impl HeightPageReader {
    pub fn new(path: PathBuf) -> HeightPageReader {
        HeightPageReader { path }
    }

    pub fn finest_level(&self) -> anyhow::Result<u32> {
        Ok(self.reader()?.header().level_count - 1)
    }

    // None when the file has no page at `address`.
    pub fn read(&self, address: PageAddress) -> anyhow::Result<Option<HeightPage>> {
        let mut reader = self.reader()?;

        if address.level >= reader.header().level_count {
            return Ok(None);
        }

        let Some(entry) = reader.entry(address)? else {
            return Ok(None);
        };
        let range = entry.range.context("a page of heights has a range")?;
        let values = reader
            .page(&entry)
            .with_context(|| format!("{}: reading {address:?}", self.path.display()))?;

        Ok(Some(HeightPage {
            range,
            values: Arc::from(values),
        }))
    }

    fn reader(&self) -> anyhow::Result<TerrainReader<BufReader<File>>> {
        let path = &self.path;
        let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let reader = TerrainReader::new(BufReader::new(file)).with_context(|| format!("reading {}", path.display()))?;
        let header = reader.header();
        ensure!(
            header.format == TextureFormat::R16Unorm && header.encoding == PageEncoding::PageHeights,
            "{}: heights must be R16_UNORM page heights, not {:?} {:?}",
            path.display(),
            header.format,
            header.encoding
        );
        ensure!(header.level_count > 0, "{}: the file has no levels", path.display());

        Ok(reader)
    }
}
