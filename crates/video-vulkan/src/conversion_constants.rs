// Must match PushConstants in shaders/color_conversion.slang.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ConversionConstants {
    pub source_extent: [i32; 2],
    pub encode_transfer: u32,
}

impl ConversionConstants {
    pub fn bytes(&self) -> &[u8] {
        // A plain struct of integers, read as the bytes Vulkan copies into push constants.
        unsafe {
            std::slice::from_raw_parts(
                (self as *const ConversionConstants).cast(),
                size_of::<ConversionConstants>(),
            )
        }
    }
}
