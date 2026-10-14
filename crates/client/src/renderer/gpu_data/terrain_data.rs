use super::GpuData;

// Heights are in radii, relative to the radius: the shader displaces the unit sphere.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TerrainData {
    pub height_texture: u32,
    pub minimum_height: f32,
    pub height_range: f32,
    pub padding: u32,
}

const _: () = assert!(size_of::<TerrainData>() == 16);

unsafe impl GpuData for TerrainData {}
