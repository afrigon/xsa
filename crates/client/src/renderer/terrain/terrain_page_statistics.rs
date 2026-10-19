use resource_vulkan::SetStatistics;

use super::TerrainLayer;

// The streamed sets' counts of every terrain drawn, heights apart and each layer by `TerrainLayer` index.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerrainPageStatistics {
    pub heights: SetStatistics,
    pub layers: [SetStatistics; TerrainLayer::ALL.len()],
}

impl TerrainPageStatistics {
    pub fn add(&mut self, other: &TerrainPageStatistics) {
        TerrainPageStatistics::add_set(&mut self.heights, &other.heights);

        for (sum, set) in self.layers.iter_mut().zip(&other.layers) {
            TerrainPageStatistics::add_set(sum, set);
        }
    }

    fn add_set(sum: &mut SetStatistics, set: &SetStatistics) {
        sum.resident += set.resident;
        sum.pending += set.pending;
        sum.ram_bytes += set.ram_bytes;
        sum.loads += set.loads;
        sum.evictions += set.evictions;
    }
}
