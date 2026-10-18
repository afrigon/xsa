use ash::vk;

use crate::{ImageLayers, QueueSharing};

pub struct ImageDescription<'a> {
    pub name: &'a str,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    /// The aspect the image's views see.
    pub aspect: vk::ImageAspectFlags,
    pub mip_levels: u32,
    pub layers: ImageLayers,
    pub sharing: QueueSharing,
}
