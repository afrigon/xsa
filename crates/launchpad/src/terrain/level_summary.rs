use resource_loader::terrain::{HeightRange, PAGE_INTERIOR, PageEntry};
use resource_loader::utils::CubeFace;

/// One level of a `.terrain` file, summed over its six faces.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LevelSummary {
    pub level: u32,
    pub present: u64,
    pub compressed_bytes: u64,
    pub range: Option<HeightRange>,
}

impl LevelSummary {
    pub fn new(level: u32) -> LevelSummary {
        LevelSummary {
            level,
            present: 0,
            compressed_bytes: 0,
            range: None,
        }
    }

    pub fn add(&mut self, entries: &[PageEntry]) {
        self.present += entries.len() as u64;
        self.compressed_bytes += entries.iter().map(|entry| u64::from(entry.data_size)).sum::<u64>();
        let bounds = entries
            .iter()
            .filter_map(|entry| entry.range)
            .flat_map(|range| [range.minimum, range.maximum]);
        self.range = HeightRange::spanning(
            self.range
                .into_iter()
                .flat_map(|range| [range.minimum, range.maximum])
                .chain(bounds),
        );
    }

    /// The width and height of each face at this level, in texels.
    pub fn face_extent(&self) -> u64 {
        u64::from(PAGE_INTERIOR) << self.level
    }

    /// The pages a complete level holds over its six faces.
    pub fn expected(&self) -> u64 {
        CubeFace::ALL.len() as u64 * Self::pages_per_face(self.level)
    }

    pub fn pages_per_face(level: u32) -> u64 {
        1u64 << (2 * level)
    }
}
