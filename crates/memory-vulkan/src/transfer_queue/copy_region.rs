use ash::vk;

// The GPU side of one copy.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CopyRegion {
    Image {
        mip_level: u32,
        array_layer: u32,
        offset: vk::Offset2D,
        extent: vk::Extent2D,
    },
    // A byte offset into the destination of an upload or the source of a readback.
    Buffer {
        offset: u64,
    },
}
