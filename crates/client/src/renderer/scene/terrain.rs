use std::sync::Arc;

use super::{MaterialHandle, ObjectHandle};
use crate::renderer::{BiomeSurfaces, TerrainMaps};

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    // None for a smooth body without maps.
    pub maps: Option<Arc<TerrainMaps>>,
    // None for ground without surfaces up close.
    pub surfaces: Option<Arc<BiomeSurfaces>>,
}
