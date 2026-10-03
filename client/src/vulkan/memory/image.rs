use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocation;

use super::{Allocator, ImageDescription};
use crate::vulkan::Device;

pub struct Image {
    image: vk::Image,
    view: vk::ImageView,
    allocation: Option<Allocation>,
}

impl Image {
    pub fn new(device: &Device, allocator: &mut Allocator, description: &ImageDescription) -> anyhow::Result<Self> {
        let device = device.handle();
        let layers = if description.cube { 6 } else { 1 };
        let flags = if description.cube {
            vk::ImageCreateFlags::CUBE_COMPATIBLE
        } else {
            vk::ImageCreateFlags::empty()
        };
        let view_type = if description.cube {
            vk::ImageViewType::CUBE
        } else {
            vk::ImageViewType::TYPE_2D
        };
        let create_info = vk::ImageCreateInfo::default()
            .flags(flags)
            .image_type(vk::ImageType::TYPE_2D)
            .format(description.format)
            .extent(description.extent.into())
            .mip_levels(description.mip_levels)
            .array_layers(layers)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(description.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = unsafe { device.create_image(&create_info, None) }?;
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        let allocation = allocator.allocate(description.name, requirements, MemoryLocation::GpuOnly, false)?;
        unsafe { device.bind_image_memory(image, allocation.memory(), allocation.offset()) }?;

        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(view_type)
            .format(description.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(description.aspect)
                    .level_count(description.mip_levels)
                    .layer_count(layers),
            );
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        Ok(Self {
            image,
            view,
            allocation: Some(allocation),
        })
    }

    pub fn handle(&self) -> vk::Image {
        self.image
    }

    pub fn view(&self) -> vk::ImageView {
        self.view
    }

    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        let device = device.handle();
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }
        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation);
        }
    }
}
