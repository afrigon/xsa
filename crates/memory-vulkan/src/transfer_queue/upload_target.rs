use ash::vk;

// The resource an upload writes.
#[derive(Clone, Copy, Debug)]
pub(crate) enum UploadTarget {
    Image {
        image: vk::Image,
        range: vk::ImageSubresourceRange,
    },
    Buffer(vk::Buffer),
}
