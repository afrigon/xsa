use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct BloomPushConstants {
    pub source_texture: u32,
    pub source_level: u32,
    pub target_image: u32,
    pub karis_average: u32,
}

unsafe impl GpuData for BloomPushConstants {}
