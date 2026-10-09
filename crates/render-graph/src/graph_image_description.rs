use ash::vk;

use crate::ImageSize;

/// An image the graph allocates. Its usage flags follow from how passes declare they use it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphImageDescription {
    pub name: &'static str,
    pub format: vk::Format,
    pub size: ImageSize,
    pub mip_levels: u32,
}

impl GraphImageDescription {
    pub(crate) fn aspect(&self) -> vk::ImageAspectFlags {
        match self.format {
            vk::Format::D16_UNORM | vk::Format::X8_D24_UNORM_PACK32 | vk::Format::D32_SFLOAT => {
                vk::ImageAspectFlags::DEPTH
            }
            vk::Format::D16_UNORM_S8_UINT | vk::Format::D24_UNORM_S8_UINT | vk::Format::D32_SFLOAT_S8_UINT => {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            }
            vk::Format::S8_UINT => vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::COLOR,
        }
    }
}
