use super::{MaterialHandle, ObjectHandle, TerrainHeight};

pub struct Terrain {
    pub object: ObjectHandle,
    pub material: MaterialHandle,
    pub height: Option<TerrainHeight>,
}

impl Terrain {
    pub fn minimum_height(&self) -> f64 {
        self.height.as_ref().map_or(0.0, |height| height.minimum)
    }

    pub fn maximum_height(&self) -> f64 {
        self.height.as_ref().map_or(0.0, |height| height.maximum)
    }
}
