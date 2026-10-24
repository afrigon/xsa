use ash::vk;

// The GPU side of one copy.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CopyRegion {
    Image {
        mip_level: u32,
        array_layer: u32,
        offset: vk::Offset3D,
        extent: vk::Extent2D,
    },
    // A region of one image copied into another, on the GPU.
    ImageToImage {
        source_offset: vk::Offset2D,
        destination_offset: vk::Offset2D,
        extent: vk::Extent2D,
    },
    // A byte offset into the destination of an upload or the source of a readback.
    Buffer {
        offset: u64,
    },
}
