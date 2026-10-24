use ash::vk;

/// One region copied from an image to another, both at their first level and layer.
#[derive(Clone, Copy, Debug)]
pub struct RegionCopy {
    pub source_offset: vk::Offset2D,
    pub destination_offset: vk::Offset2D,
    pub extent: vk::Extent2D,
}
