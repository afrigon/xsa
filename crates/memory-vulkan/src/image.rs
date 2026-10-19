use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationScheme};

use crate::{DeviceObject, ImageDescription, ImageLayers, ImageMemory, MemoryContext, MemoryError};

/// An image in GPU memory and a view of all its levels and layers.
pub struct Image {
    // Declaration order is destruction order: the view, then the image, then its memory.
    view: DeviceObject<vk::ImageView>,
    image: DeviceObject<vk::Image>,
    allocation: DeviceObject<Allocation>,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
    mip_levels: u32,
    layers: ImageLayers,
}

impl Image {
    pub fn new(memory: &MemoryContext, description: &ImageDescription) -> Result<Image, MemoryError> {
        let device = memory.device();
        let create_info = vk::ImageCreateInfo::default()
            .flags(description.layers.create_flags())
            .image_type(vk::ImageType::TYPE_2D)
            .format(description.format)
            .extent(description.extent.into())
            .mip_levels(description.mip_levels)
            .array_layers(description.layers.count())
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(description.usage)
            .sharing_mode(description.sharing.mode())
            .queue_family_indices(description.sharing.families())
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = DeviceObject::new(memory, unsafe { device.create_image(&create_info, None) }?);
        let requirements = unsafe { device.get_image_memory_requirements(image.handle()) };
        let allocation_scheme = match description.memory {
            ImageMemory::Shared => AllocationScheme::GpuAllocatorManaged,
            ImageMemory::Dedicated => AllocationScheme::DedicatedImage(image.handle()),
        };
        let allocation = memory.allocate(
            description.name,
            requirements,
            MemoryLocation::GpuOnly,
            false,
            allocation_scheme,
        )?;
        unsafe { device.bind_image_memory(image.handle(), allocation.get().memory(), allocation.get().offset()) }?;
        let view = Image::create_view_of(
            memory,
            image.handle(),
            description.format,
            description.aspect,
            description.layers,
            0,
            description.mip_levels,
        )?;

        Ok(Image {
            view,
            image,
            allocation,
            format: description.format,
            aspect: description.aspect,
            mip_levels: description.mip_levels,
            layers: description.layers,
        })
    }

    pub fn handle(&self) -> vk::Image {
        self.image.handle()
    }

    /// The bytes of memory it occupies.
    pub fn size(&self) -> u64 {
        self.allocation.get().size()
    }

    /// The view of every level and layer.
    pub fn view(&self) -> vk::ImageView {
        self.view.handle()
    }

    /// Every level and layer, as barriers name them.
    pub(crate) fn subresource_range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange::default()
            .aspect_mask(self.aspect)
            .level_count(self.mip_levels)
            .layer_count(self.layers.count())
    }

    /// Another view of `level_count` levels from `base_level`, owned by the caller and dropped before the image.
    pub fn create_view(
        &self,
        memory: &MemoryContext,
        base_level: u32,
        level_count: u32,
    ) -> Result<DeviceObject<vk::ImageView>, MemoryError> {
        Image::create_view_of(
            memory,
            self.image.handle(),
            self.format,
            self.aspect,
            self.layers,
            base_level,
            level_count,
        )
    }

    fn create_view_of(
        memory: &MemoryContext,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
        layers: ImageLayers,
        base_level: u32,
        level_count: u32,
    ) -> Result<DeviceObject<vk::ImageView>, MemoryError> {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(layers.view_type())
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(base_level)
                    .level_count(level_count)
                    .layer_count(layers.count()),
            );
        let view = unsafe { memory.device().create_image_view(&view_info, None) }?;

        Ok(DeviceObject::new(memory, view))
    }
}
