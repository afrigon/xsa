use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocation;

use crate::{DeviceObject, ImageDescription, MemoryContext, MemoryError};

const CUBE_FACES: u32 = 6;

/// An image in GPU memory and a view of all its levels and layers.
pub struct Image {
    // Declaration order is destruction order: the view, then the image, then its memory.
    view: DeviceObject<vk::ImageView>,
    image: DeviceObject<vk::Image>,
    _allocation: DeviceObject<Allocation>,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
    cube: bool,
}

impl Image {
    pub fn new(memory: &MemoryContext, description: &ImageDescription) -> Result<Image, MemoryError> {
        let device = memory.device();
        let (flags, layers) = if description.cube {
            (vk::ImageCreateFlags::CUBE_COMPATIBLE, CUBE_FACES)
        } else {
            (vk::ImageCreateFlags::empty(), 1)
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
            .sharing_mode(description.sharing.mode())
            .queue_family_indices(description.sharing.families())
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = DeviceObject::new(memory, unsafe { device.create_image(&create_info, None) }?);
        let requirements = unsafe { device.get_image_memory_requirements(image.handle()) };
        let allocation = memory.allocate(description.name, requirements, MemoryLocation::GpuOnly, false)?;
        unsafe { device.bind_image_memory(image.handle(), allocation.get().memory(), allocation.get().offset()) }?;
        let view = Image::create_view_of(
            memory,
            image.handle(),
            description.format,
            description.aspect,
            description.cube,
            0,
            description.mip_levels,
        )?;

        Ok(Image {
            view,
            image,
            _allocation: allocation,
            format: description.format,
            aspect: description.aspect,
            cube: description.cube,
        })
    }

    pub fn handle(&self) -> vk::Image {
        self.image.handle()
    }

    /// The view of every level and layer.
    pub fn view(&self) -> vk::ImageView {
        self.view.handle()
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
            self.cube,
            base_level,
            level_count,
        )
    }

    fn create_view_of(
        memory: &MemoryContext,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
        cube: bool,
        base_level: u32,
        level_count: u32,
    ) -> Result<DeviceObject<vk::ImageView>, MemoryError> {
        let (view_type, layers) = if cube {
            (vk::ImageViewType::CUBE, CUBE_FACES)
        } else {
            (vk::ImageViewType::TYPE_2D, 1)
        };
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(view_type)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect)
                    .base_mip_level(base_level)
                    .level_count(level_count)
                    .layer_count(layers),
            );
        let view = unsafe { memory.device().create_image_view(&view_info, None) }?;

        Ok(DeviceObject::new(memory, view))
    }
}
