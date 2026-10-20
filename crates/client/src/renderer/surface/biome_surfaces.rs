use glam::{DVec2, DVec3, Vec2};

use super::{SURFACE_CLASSES, SurfaceTextures, TRIPLANAR_PLANES};
use crate::renderer::gpu_data::{MAX_SURFACE_SCALES, SurfaceAnchorData, SurfaceData};

// A body's biome as its ground draws up close: a surface per class of ground.
pub struct BiomeSurfaces {
    classes: [SurfaceTextures; SURFACE_CLASSES],
}

impl BiomeSurfaces {
    pub fn new(classes: [SurfaceTextures; SURFACE_CLASSES]) -> BiomeSurfaces {
        BiomeSurfaces { classes }
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

        SurfaceAnchorData { planes }
    }
}
