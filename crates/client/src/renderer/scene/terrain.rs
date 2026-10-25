use std::sync::Arc;

use super::{MaterialHandle, ObjectHandle};
use crate::renderer::{BiomeSurfaces, TerrainPages};

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    // None for a smooth body without layers.
    pub pages: Option<Arc<TerrainPages>>,
    // None for ground without surfaces up close.
    pub surfaces: Option<Arc<BiomeSurfaces>>,
}
