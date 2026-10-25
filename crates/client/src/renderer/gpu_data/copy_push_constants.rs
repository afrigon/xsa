use super::GpuData;

// Must match CopyConstants in post/copy.slang.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct CopyPushConstants {
    pub source_texture: u32,
}

const _: () = assert!(size_of::<CopyPushConstants>() == 4);

unsafe impl GpuData for CopyPushConstants {}
