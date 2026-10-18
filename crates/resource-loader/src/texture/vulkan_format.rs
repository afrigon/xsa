/// A `VkFormat` value, as Vulkan and the file formats that name Vulkan formats store it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VulkanFormat {
    pub value: u32,
}
