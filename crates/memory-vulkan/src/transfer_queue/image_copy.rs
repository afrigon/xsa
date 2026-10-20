use ash::vk;

/// One region of an image written from tightly packed rows of texel blocks.
#[derive(Clone, Copy, Debug)]
pub struct ImageCopy {
    /// The index of the byte array in the upload's `data` holding the region.
    pub data: usize,
    /// Where the region starts in that array, in bytes.
    pub offset: usize,
    pub mip_level: u32,
    pub array_layer: u32,
    /// Where the region starts in the image; z is the depth slice of a volume, 0 otherwise.
    pub image_offset: vk::Offset3D,
    pub extent: vk::Extent2D,
}
