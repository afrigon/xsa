use std::collections::HashMap;
use std::sync::Mutex;

use resource_loader::terrain::{HeightRange, PageAddress};

// The range of every height page read so far, kept after its bytes are evicted: a few bytes per page, and what both
// the shader and tile bounds need for a resident page.
#[derive(Default)]
pub(in crate::renderer) struct PageRanges {
    ranges: Mutex<HashMap<PageAddress, HeightRange>>,
}

impl PageRanges {
    pub fn record(&self, address: PageAddress, range: HeightRange) {
        self.lock().insert(address, range);
    }

    pub fn get(&self, address: PageAddress) -> Option<HeightRange> {
        self.lock().get(&address).copied()
    }

    // The range of `address`, or of its nearest coarser page read, with that page's address.
    pub fn nearest(&self, address: PageAddress) -> Option<(PageAddress, HeightRange)> {
        let ranges = self.lock();
        let mut current = Some(address);

        while let Some(address) = current {
            if let Some(&range) = ranges.get(&address) {
                return Some((address, range));
            }

            current = address.coarser();
        }

        None
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<PageAddress, HeightRange>> {
        self.ranges.lock().expect("the page ranges' lock is poisoned")
    }
}
