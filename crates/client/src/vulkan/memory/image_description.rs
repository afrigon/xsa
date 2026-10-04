use ash::vk;

pub struct ImageDescription<'a> {
    pub name: &'a str,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub aspect: vk::ImageAspectFlags,
    pub mip_levels: u32,
    pub cube: bool,
}
