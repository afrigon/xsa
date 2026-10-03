use anyhow::Context;
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocation;

use super::Allocator;
use crate::vulkan::Device;

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
