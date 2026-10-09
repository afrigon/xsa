use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};

use crate::access::Access;
use crate::image_owner::ImageOwner;
use crate::{GraphImageDescription, ImageState, RenderGraphError};

pub(crate) struct PhysicalImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub level_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub description: GraphImageDescription,
    pub usage: vk::ImageUsageFlags,
    pub owner: ImageOwner,
    pub states: Vec<ImageState>,
    allocation: Option<Allocation>,
}

impl PhysicalImage {
    pub fn new(
        device: &ash::Device,
        allocator: &mut Allocator,
        description: GraphImageDescription,
        usage: vk::ImageUsageFlags,
        output: vk::Extent2D,
        owner: ImageOwner,
    ) -> Result<PhysicalImage, RenderGraphError> {
        let extent = description.size.extent(output);
        let create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(description.format)
            .extent(extent.into())
            .mip_levels(description.mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = unsafe { device.create_image(&create_info, None) }?;
        let mut physical = PhysicalImage {
            image,
            view: vk::ImageView::null(),
            level_views: Vec::new(),
            extent,
            description,
            usage,
            owner,
            states: vec![ImageState::UNUSED; description.mip_levels as usize],
            allocation: None,
        };

        if let Err(error) = physical.bind_memory_and_views(device, allocator) {
            // The creation error is the one worth reporting; a failed cleanup adds nothing to it.
            let _ = unsafe { physical.destroy(device, allocator) };
            return Err(error);
        }

        Ok(physical)
    }

    pub fn level_view(&self, level: u32) -> vk::ImageView {
        if self.level_views.is_empty() {
            self.view
        } else {
            self.level_views[level as usize]
        }
    }

    pub fn sampled_layout(&self) -> vk::ImageLayout {
        Access::sampled_layout(self.usage)
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        unsafe {
            for view in self.level_views.drain(..) {
                device.destroy_image_view(view, None);
            }

            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }

        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation)?;
        }

        Ok(())
    }

    fn bind_memory_and_views(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
    ) -> Result<(), RenderGraphError> {
        let requirements = unsafe { device.get_image_memory_requirements(self.image) };
        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: self.description.name,
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        let bound = unsafe { device.bind_image_memory(self.image, allocation.memory(), allocation.offset()) };
        self.allocation = Some(allocation);
        bound?;
        self.view = self.create_view(device, 0, self.description.mip_levels)?;

        if self.description.mip_levels > 1 {
            for level in 0..self.description.mip_levels {
                let view = self.create_view(device, level, 1)?;
                self.level_views.push(view);
            }
        }

        Ok(())
    }

    fn create_view(
        &self,
        device: &ash::Device,
        base_level: u32,
        level_count: u32,
    ) -> Result<vk::ImageView, RenderGraphError> {
        let aspect = self.description.aspect();
        let view_aspect = if aspect.contains(vk::ImageAspectFlags::DEPTH) {
            vk::ImageAspectFlags::DEPTH
        } else {
            aspect
        };
        let view_info = vk::ImageViewCreateInfo::default()
            .image(self.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(self.description.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(view_aspect)
                    .base_mip_level(base_level)
                    .level_count(level_count)
                    .layer_count(1),
            );

        Ok(unsafe { device.create_image_view(&view_info, None) }?)
    }
}
