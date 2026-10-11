use super::GpuData;

// One lens ghost: a texture centered on the line from the light through the screen center.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct GhostData {
    pub position: f32,
    // Angular diameter, in radians.
    pub size: f32,
    pub texture: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<GhostData>() == 16);

unsafe impl GpuData for GhostData {}
