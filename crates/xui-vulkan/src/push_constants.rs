// Must match PushConstants in shaders/xui.slang.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PushConstants {
    pub viewport_size: [f32; 2],
    pub atlas_size: [f32; 2],
}

const _: () = assert!(size_of::<PushConstants>() == 16);

impl PushConstants {
    pub fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const PushConstants).cast::<u8>(), size_of::<PushConstants>()) }
    }
}
