use ash::vk;

/// The smallest unit of an image format: one texel, or one compressed block of texels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TexelBlock {
    pub extent: vk::Extent2D,
    /// In bytes.
    pub size: u32,
}

impl TexelBlock {
    /// The bytes of one row of blocks across `width` texels.
    pub(crate) fn row_size(&self, width: u32) -> usize {
        width.div_ceil(self.extent.width) as usize * self.size as usize
    }

    pub(crate) fn rows(&self, height: u32) -> u32 {
        height.div_ceil(self.extent.height)
    }
}
