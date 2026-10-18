use ash::vk;

// The resource an upload writes.
#[derive(Clone, Copy, Debug)]
pub(crate) enum UploadTarget {
    Image {
        image: vk::Image,
        range: vk::ImageSubresourceRange,
        // The layout shaders read it in once written.
        layout: vk::ImageLayout,
    },
    Buffer(vk::Buffer),
}
