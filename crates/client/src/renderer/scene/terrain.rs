use resource_vulkan::ResourceHandle;

use super::{MaterialHandle, ObjectHandle};
use crate::resource_kinds::TerrainHeightMap;

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    // None for a smooth body; drawn smooth until it loads.
    pub height_map: Option<ResourceHandle<TerrainHeightMap>>,
}
