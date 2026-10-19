use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, ensure};
use resource_loader::terrain::{PageAddress, PageEncoding, TerrainReader};
use resource_loader::texture::TextureFormat;
use xsa_core::terrain::HeightPage;

// A body's height pages in `data/terrains/<body>.terrain`. Each read opens the file anew, so readers on several
// threads never share a position in it.
pub struct HeightPageReader {
    path: PathBuf,
    finest_level: u32,
}

impl HeightPageReader {
    pub fn open(path: &Path) -> anyhow::Result<HeightPageReader> {
        let reader = HeightPageReader::reader(path)?;
        let header = reader.header();
        ensure!(
            header.format == TextureFormat::R16Unorm && header.encoding == PageEncoding::PageHeights,
            "{}: heights must be R16_UNORM page heights, not {:?} {:?}",
            path.display(),
            header.format,
            header.encoding
        );
        ensure!(header.level_count > 0, "{}: the file has no levels", path.display());

        Ok(HeightPageReader {
            path: path.to_path_buf(),
            finest_level: header.level_count - 1,
        })
    }

    pub fn finest_level(&self) -> u32 {
        self.finest_level
    }

    // None when the file has no page at `address`.
    pub fn read(&self, address: PageAddress) -> anyhow::Result<Option<HeightPage>> {
        let mut reader = HeightPageReader::reader(&self.path)?;
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

    fn reader(path: &Path) -> anyhow::Result<TerrainReader<BufReader<File>>> {
        let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;

        TerrainReader::new(BufReader::new(file)).with_context(|| format!("reading {}", path.display()))
    }
}
