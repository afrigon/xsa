use std::sync::Arc;

use super::{MaterialHandle, ObjectHandle};
use crate::renderer::TerrainPages;

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    // None for a smooth body without layers.
    pub pages: Option<Arc<TerrainPages>>,
}
