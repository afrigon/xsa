use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Context;
use resource_loader::terrain::{PageAddress, TerrainReader};
use resource_vulkan::PageSource;

// Reads a shading layer's pages. Each read opens the file anew, so workers never share a position in it.
pub(in crate::renderer) struct LayerPageSource {
    pub path: PathBuf,
}

impl PageSource for LayerPageSource {
    type Key = PageAddress;

    fn coarser(&self, key: PageAddress) -> Option<PageAddress> {
        key.coarser()
    }

    fn read(&self, key: PageAddress) -> anyhow::Result<Option<Vec<u8>>> {
        let path = &self.path;
        let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let mut reader =
            TerrainReader::new(BufReader::new(file)).with_context(|| format!("reading {}", path.display()))?;

        if key.level >= reader.header().level_count {
            return Ok(None);
        }

        let Some(entry) = reader.entry(key)? else {
            return Ok(None);
        };

        reader.page(&entry).map(Some)
    }
}
