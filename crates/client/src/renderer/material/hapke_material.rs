use std::sync::Arc;

use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::hapke_data::HapkeData;
use super::{Material, MaterialData, texture_index};
use crate::renderer::BiomeSurfaces;
use crate::renderer::gpu_data::SurfaceData;
use crate::renderer::surface::SURFACE_CLASSES;

// A terrain shaded with Hapke 2012. Its albedo, normals and Hapke parameters are terrain layers the tiles read; where
// a tile has no Hapke parameters, it is shaded with Lambert. Up close, its biome's surfaces add to the layers.
pub struct HapkeMaterial {
    pub base_color: Vec3,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
    pub surfaces: Option<Arc<BiomeSurfaces>>,
}

impl Material for HapkeMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        let surfaces = self.surfaces.as_deref().and_then(BiomeSurfaces::gpu_data);

        Some(MaterialData::new(HapkeData {
            base_color: self.base_color,
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            surfaces_loaded: u32::from(surfaces.is_some()),
            padding: [0; 2],
            surfaces: surfaces.unwrap_or([SurfaceData::NONE; SURFACE_CLASSES]),
        }))
    }
}
