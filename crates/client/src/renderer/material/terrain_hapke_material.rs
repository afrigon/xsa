use std::sync::Arc;

use glam::Vec3;
use resource_vulkan::{ResourceHandle, Texture};

use super::terrain_hapke_data::TerrainHapkeData;
use super::{Material, MaterialData, texture_index};
use crate::renderer::BiomeSurfaces;
use crate::renderer::gpu_data::{BiomeData, SurfaceData};
use crate::renderer::surface::SURFACE_CLASSES;

// A terrain shaded with Hapke 2012. Its albedo, normals and the Hapke parameters varying over the ground are terrain
// layers the tiles read; without both Hapke layers, it is shaded with Lambert. The macroscopic roughness (radians) and
// porosity coefficient hold over the whole body. Up close, its biome's surfaces add to the layers.
pub struct TerrainHapkeMaterial {
    pub base_color: Vec3,
    pub emissive: Option<ResourceHandle<Texture>>,
    pub emissive_luminance: f32,
    pub macroscopic_roughness: f32,
    pub porosity: f32,
    pub surfaces: Option<Arc<BiomeSurfaces>>,
}

impl Material for TerrainHapkeMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        let surfaces = self.surfaces.as_deref().and_then(BiomeSurfaces::gpu_data);

        Some(MaterialData::new(TerrainHapkeData {
            base_color: self.base_color,
            emissive_texture: texture_index(self.emissive.as_ref()),
            emissive_luminance: self.emissive_luminance,
            surfaces_loaded: u32::from(surfaces.is_some()),
            macroscopic_roughness: self.macroscopic_roughness,
            porosity: self.porosity,
            biome: self
                .surfaces
                .as_deref()
                .map_or(BiomeData::NONE, BiomeSurfaces::biome_data),
            surfaces: surfaces.unwrap_or([SurfaceData::NONE; SURFACE_CLASSES]),
        }))
    }
}
