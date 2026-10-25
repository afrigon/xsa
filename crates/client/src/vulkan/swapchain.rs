use anyhow::Context;
use ash::vk;
use memory_vulkan::{DeviceObject, MemoryContext};

use super::{Device, Surface};

const PREFERRED_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
    format: vk::Format::B8G8R8A8_SRGB,
    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
};

// Replacing it queues the old one's destruction behind its last presents.
pub struct Swapchain {
    // Declaration order is destruction order: the semaphores and views before the swapchain.
    render_finished: Vec<DeviceObject<vk::Semaphore>>,
    image_views: Vec<DeviceObject<vk::ImageView>>,
    swapchain: DeviceObject<vk::SwapchainKHR>,
    images: Vec<vk::Image>,
    extent: vk::Extent2D,
    format: vk::Format,
}

impl Swapchain {
    pub fn new(
        memory: &MemoryContext,
        surface: &Surface,
        device: &Device,
        window_extent: vk::Extent2D,
        old_swapchain: vk::SwapchainKHR,
    ) -> anyhow::Result<Self> {
        let physical_device = device.physical_device();
        let capabilities = unsafe {
            surface
                .loader()
                .get_physical_device_surface_capabilities(physical_device, surface.handle())
        }?;
        let formats = unsafe {
            surface
                .loader()
                .get_physical_device_surface_formats(physical_device, surface.handle())
        }?;
        let format = formats
            .iter()
            .copied()
            .find(|format| *format == PREFERRED_FORMAT)
            .or(formats.first().copied())
            .context("the surface reports no formats")?;

        let extent = if capabilities.current_extent.width == u32::MAX {
            vk::Extent2D {
                width: window_extent
                    .width
                    .clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                height: window_extent.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        } else {
            capabilities.current_extent
        };

        let mut image_count = capabilities.min_image_count + 1;

        if capabilities.max_image_count > 0 {
            image_count = image_count.min(capabilities.max_image_count);
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.handle())
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(old_swapchain);

        let device = memory.device();
        let loader = memory.swapchain_loader();
        let swapchain = DeviceObject::new(
            memory,
            unsafe { loader.create_swapchain(&create_info, None) }.context("creating the swapchain")?,
        );
        let images = unsafe { loader.get_swapchain_images(swapchain.handle()) }?;

        let mut image_views = Vec::with_capacity(images.len());
        let mut render_finished = Vec::with_capacity(images.len());

        for &image in &images {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                );
            image_views.push(DeviceObject::new(memory, unsafe {
                device.create_image_view(&view_info, None)
            }?));
            render_finished.push(DeviceObject::new(memory, unsafe {
                device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            }?));
        }

        Ok(Self {
            render_finished,
            image_views,
            swapchain,
            images,
            extent,
            format: format.format,
        })
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.swapchain.handle()
    }

    pub fn image(&self, index: u32) -> vk::Image {
        self.images[index as usize]
    }

    pub fn image_view(&self, index: u32) -> vk::ImageView {
        self.image_views[index as usize].handle()
    }

    pub fn render_finished(&self, index: u32) -> vk::Semaphore {
        self.render_finished[index as usize].handle()
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }
}
