use std::sync::Arc;

use xsa_core::terrain::{EquirectangularGrid, HeightMap};

use super::quad_sphere::HeightBounds;

// A terrain's heights on the CPU: the bounds tile selection culls with, and the map they come from.
pub(in crate::renderer) struct TerrainHeight {
    pub bounds: HeightBounds,
    pub map: Arc<HeightMap>,
}

impl TerrainHeight {
    pub fn new(map: Arc<HeightMap>) -> TerrainHeight {
        TerrainHeight {
            bounds: HeightBounds::new(&map),
            map,
        }
    }

    // Meters relative to the body's radius.
    pub fn maximum(&self) -> f64 {
        self.bounds.maximum()
    }

    pub fn grid(&self) -> EquirectangularGrid {
        self.map.grid()
    }
}
