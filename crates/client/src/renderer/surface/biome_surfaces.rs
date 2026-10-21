use glam::{DVec2, DVec3, Vec2};
use xsa_core::random::Seed;
use xsa_packs::BiomeSurfacesDefinition;

use super::{PROJECTION_AXES, SURFACE_CLASSES, SURFACE_LEVELS, SurfaceTextures};
use crate::renderer::gpu_data::{BiomeData, NoiseAnchor, SurfaceAnchorData, SurfaceData};

// A body's biome as its ground draws up close: a surface per class of ground, where the classes hand over, and the
// seeded noise (m) moving the handovers.
pub struct BiomeSurfaces {
    classes: [SurfaceTextures; SURFACE_CLASSES],
    biome: BiomeData,
    handover_noise: f64,
    handover_seed: Seed,
}

impl BiomeSurfaces {
    pub fn new(
        classes: [SurfaceTextures; SURFACE_CLASSES],
        definition: &BiomeSurfacesDefinition,
        seed: Seed,
    ) -> BiomeSurfaces {
        BiomeSurfaces {
            classes,
            biome: BiomeData {
                transition_angle: definition.transition.angle,
                transition_width: definition.transition.width,
                handover_noise_amplitude: definition.handover_noise.amplitude,
                convex_bias: definition.curvature.convex,
                concave_bias: definition.curvature.concave,
                relief_bias: definition.curvature.relief,
                padding: [0; 2],
            },
            handover_noise: f64::from(definition.handover_noise.size),
            handover_seed: seed.derive("handovers"),
        }
    }

    pub(in crate::renderer) fn biome_data(&self) -> BiomeData {
        self.biome
    }

    // None until every class's arrays are loaded.
    pub(in crate::renderer) fn gpu_data(&self) -> Option<[SurfaceData; SURFACE_CLASSES]> {
        let mut data = [SurfaceData::NONE; SURFACE_CLASSES];

        for (slot, class) in data.iter_mut().zip(&self.classes) {
            *slot = class.gpu_data()?;
        }

        Some(data)
    }

    // Plane p projects along the body frame's axis p, its coordinates the two axes after it.
    pub(in crate::renderer) fn anchor_data(&self, camera_in_body: DVec3, noise_volume: u32) -> SurfaceAnchorData {
        let mut repeats = [[[Vec2::ZERO; SURFACE_LEVELS]; PROJECTION_AXES]; SURFACE_CLASSES];

        for (class_repeats, class) in repeats.iter_mut().zip(&self.classes) {
            for (plane, levels) in class_repeats.iter_mut().enumerate() {
                let point = DVec2::new(
                    camera_in_body[(plane + 1) % PROJECTION_AXES],
                    camera_in_body[(plane + 2) % PROJECTION_AXES],
                );

                for (level, place) in levels.iter_mut().enumerate() {
                    *place = class.repeat_place(point, level);
                }
            }
        }

        SurfaceAnchorData {
            repeats,
            handover_noise: NoiseAnchor::at(camera_in_body, self.handover_noise, self.handover_seed),
            noise_volume,
            padding: [0; 3],
        }
    }
}
