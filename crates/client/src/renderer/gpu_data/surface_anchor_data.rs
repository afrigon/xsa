use glam::Vec2;

use super::{GpuData, NoiseAnchor};
use crate::renderer::surface::{PROJECTION_AXES, SURFACE_CLASSES, SURFACE_LEVELS};

// Where the camera stands within each class's repeat, in repeats, for every projection plane and level. Placed in f64
// each frame, so pixels only add their small offset from the camera in f32, with the noise that moves the handover,
// read from the noise volume at bindless index `noise_volume`. Mirrored by
// `SurfaceAnchors` in terrain/surface/surface-data.slang.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct SurfaceAnchorData {
    pub repeats: [[[Vec2; SURFACE_LEVELS]; PROJECTION_AXES]; SURFACE_CLASSES],
    pub handover_noise: NoiseAnchor,
    pub noise_volume: u32,
    pub padding: [u32; 3],
}

const _: () = assert!(size_of::<SurfaceAnchorData>() == 608);

unsafe impl GpuData for SurfaceAnchorData {}
