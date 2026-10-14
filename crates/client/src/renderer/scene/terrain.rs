use super::{MaterialHandle, ObjectHandle, TerrainHeight};

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    pub height: Option<TerrainHeight>,
}
