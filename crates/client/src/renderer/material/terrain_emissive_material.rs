use glam::Vec3;

use super::terrain_emissive_data::TerrainEmissiveData;
use super::{Material, MaterialData};

pub struct TerrainEmissiveMaterial {
    pub luminance: Vec3,
}

impl Material for TerrainEmissiveMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(TerrainEmissiveData {
            luminance: self.luminance,
            padding: 0,
        }))
    }
}
