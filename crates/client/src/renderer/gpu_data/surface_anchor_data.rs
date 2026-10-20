use glam::Vec2;

use super::{GpuData, MAX_SURFACE_SCALES};
use crate::renderer::surface::{SURFACE_CLASSES, TRIPLANAR_PLANES};

// Where the camera stands within each class's texture repeat, for every scale and projection plane, in repeats. Placed
// in f64 each frame, so pixels only add their small offset from the camera in f32. Mirrored by `SurfaceAnchors` in
// terrain/surface/surface-data.slang.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct SurfaceAnchorData {
    pub planes: [[[Vec2; TRIPLANAR_PLANES]; MAX_SURFACE_SCALES]; SURFACE_CLASSES],
}

const _: () = assert!(size_of::<SurfaceAnchorData>() == 432);

unsafe impl GpuData for SurfaceAnchorData {}
