use anyhow::Context;
use ash::vk;
pub use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, AllocatorCreateDesc};

use super::{Device, Instance};

pub struct Allocator {
    allocator: gpu_allocator::vulkan::Allocator,
}

impl Allocator {
    pub fn new(instance: &Instance, device: &Device) -> anyhow::Result<Self> {
        let allocator = gpu_allocator::vulkan::Allocator::new(&AllocatorCreateDesc {
            instance: instance.handle().clone(),
            device: device.handle().clone(),
            physical_device: device.physical_device(),
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .context("creating the GPU memory allocator")?;
        Ok(Self { allocator })
    }

    fn allocate(
        &mut self,
        name: &str,
        requirements: vk::MemoryRequirements,
        location: MemoryLocation,
        linear: bool,
    ) -> anyhow::Result<Allocation> {
        let allocation = self.allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        Ok(allocation)
    }

    fn free(&mut self, allocation: Allocation) {
        if let Err(err) = self.allocator.free(allocation) {
            eprintln!("freeing GPU memory: {err}");
        }
    }
}

pub struct Buffer {
    buffer: vk::Buffer,
    device_address: Option<vk::DeviceAddress>,
    allocation: Option<Allocation>,
}

impl Buffer {
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        name: &str,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> anyhow::Result<Self> {
        let device = device.handle();
        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&create_info, None) }?;
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let allocation = allocator.allocate(name, requirements, location, true)?;
        unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }?;
        let device_address = usage
            .contains(vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .then(|| unsafe {
                device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(buffer))
            });
        Ok(Self {
            buffer,
            device_address,
            allocation: Some(allocation),
        })
    }

    pub fn handle(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.device_address
            .expect("the buffer was created without SHADER_DEVICE_ADDRESS usage")
    }

    pub fn write<T: Copy>(&mut self, data: &[T]) -> anyhow::Result<()> {
        let destination = self
            .allocation
            .as_mut()
            .and_then(|allocation| allocation.mapped_slice_mut())
            .context("writing to a buffer the CPU cannot access")?;
        let size = size_of_val(data);
        anyhow::ensure!(
            size <= destination.len(),
            "writing {size} bytes past the end of a buffer"
        );
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), destination.as_mut_ptr(), size) };
        Ok(())
    }

    pub fn read<T: Copy>(&self, data: &mut [T]) -> anyhow::Result<()> {
        let source = self
            .allocation
            .as_ref()
            .and_then(|allocation| allocation.mapped_slice())
            .context("reading from a buffer the CPU cannot access")?;
        let size = size_of_val(data);
        anyhow::ensure!(size <= source.len(), "reading {size} bytes past the end of a buffer");
        unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), data.as_mut_ptr().cast::<u8>(), size) };
        Ok(())
    }

    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        unsafe { device.handle().destroy_buffer(self.buffer, None) };
        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation);
        }
    }
}

pub struct ImageDescription<'a> {
    pub name: &'a str,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub aspect: vk::ImageAspectFlags,
    pub mip_levels: u32,
    pub cube: bool,
}

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
