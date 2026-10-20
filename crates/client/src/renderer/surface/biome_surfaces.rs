use glam::{DVec2, DVec3, Vec2};
use xsa_core::random::Seed;
use xsa_packs::BiomeSurfacesDefinition;

use super::{SURFACE_CLASSES, SurfaceTextures, TRIPLANAR_PLANES};
use crate::renderer::gpu_data::{BiomeData, MAX_SURFACE_SCALES, NoiseAnchor, SurfaceAnchorData, SurfaceData};

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
                flat_to_slope_angle: definition.flat_to_slope.angle,
                flat_to_slope_width: definition.flat_to_slope.width,
                slope_to_steep_angle: definition.slope_to_steep.angle,
                slope_to_steep_width: definition.slope_to_steep.width,
                handover_noise_amplitude: definition.handover_noise.amplitude,
                convex_bias: definition.curvature.convex,
                concave_bias: definition.curvature.concave,
                relief_bias: definition.curvature.relief,
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
    pub(in crate::renderer) fn anchor_data(&self, camera_in_body: DVec3) -> SurfaceAnchorData {
        let mut planes = [[[Vec2::ZERO; TRIPLANAR_PLANES]; MAX_SURFACE_SCALES]; SURFACE_CLASSES];

        for (class_planes, class) in planes.iter_mut().zip(&self.classes) {
            for plane in 0..TRIPLANAR_PLANES {
                let point = DVec2::new(
                    camera_in_body[(plane + 1) % TRIPLANAR_PLANES],
                    camera_in_body[(plane + 2) % TRIPLANAR_PLANES],
                );

                for (scale, anchor) in class.anchors(point).into_iter().enumerate() {
                    class_planes[scale][plane] = anchor;
                }
            }
        }

        let mut variant_noise = [NoiseAnchor::NONE; SURFACE_CLASSES];

        for (anchor, class) in variant_noise.iter_mut().zip(&self.classes) {
            *anchor = class.variant_noise_anchor(camera_in_body);
        }

        SurfaceAnchorData {
            planes,
            handover_noise: NoiseAnchor::at(camera_in_body, self.handover_noise, self.handover_seed),
            variant_noise,
        }
    }
}
