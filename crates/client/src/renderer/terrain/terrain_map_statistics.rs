use super::{MapStatistics, TerrainLayer};

// The maps of every terrain drawn, heights apart and each layer by `TerrainLayer` index.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerrainMapStatistics {
    pub heights: MapStatistics,
    pub layers: [MapStatistics; TerrainLayer::ALL.len()],
}

impl TerrainMapStatistics {
    pub fn add(&mut self, other: &TerrainMapStatistics) {
        self.heights.add(other.heights);

        for (sum, layer) in self.layers.iter_mut().zip(other.layers) {
            sum.add(layer);
        }
    }
}
