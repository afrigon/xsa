// One kind of terrain map over every terrain: the memory its resident levels hold and how many still load toward
// their demand.
#[derive(Clone, Copy, Debug, Default)]
pub struct MapStatistics {
    pub video_bytes: u64,
    pub ram_bytes: u64,
    pub loading: usize,
}

impl MapStatistics {
    pub fn add(&mut self, other: MapStatistics) {
        self.video_bytes += other.video_bytes;
        self.ram_bytes += other.ram_bytes;
        self.loading += other.loading;
    }
}
