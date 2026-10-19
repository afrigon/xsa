use std::sync::Arc;

use resource_loader::terrain::PageAddress;
use resource_vulkan::PageSource;
use xsa_packs::HeightPageReader;

use super::PageRanges;

// Reads a terrain's height pages for drawing, recording each page's range for the shader and the CPU's heights.
pub(in crate::renderer) struct HeightPageSource {
    pub reader: HeightPageReader,
    pub ranges: Arc<PageRanges>,
}

impl PageSource for HeightPageSource {
    type Key = PageAddress;

    fn coarser(&self, key: PageAddress) -> Option<PageAddress> {
        key.coarser()
    }

    fn read(&self, key: PageAddress) -> anyhow::Result<Option<Vec<u8>>> {
        let Some(page) = self.reader.read(key)? else {
            return Ok(None);
        };
        self.ranges.record(key, page.range);

        Ok(Some(page.values.to_vec()))
    }
}
