use std::sync::Arc;

use xsa_core::terrain::{EquirectangularGrid, HeightMap};

use super::TerrainBufferHandle;
use crate::renderer::quad_sphere::HeightBounds;

pub struct TerrainHeight {
    pub buffer: TerrainBufferHandle,
    pub(in crate::renderer) bounds: HeightBounds,
    pub(in crate::renderer) map: Arc<HeightMap>,
}

impl TerrainHeight {
    // Meters relative to the body's radius.
    pub fn maximum(&self) -> f64 {
        self.bounds.maximum()
    }

    pub(in crate::renderer) fn grid(&self) -> EquirectangularGrid {
        self.map.grid()
    }
}
