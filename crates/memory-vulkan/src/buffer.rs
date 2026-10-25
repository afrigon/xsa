use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationScheme};

use crate::{DeviceObject, MemoryContext, MemoryError, QueueSharing};

/// A buffer and the memory bound to it.
pub struct Buffer {
    // Declaration order is destruction order: the buffer before its memory.
    buffer: DeviceObject<vk::Buffer>,
    allocation: DeviceObject<Allocation>,
    device_address: Option<vk::DeviceAddress>,
    size: u64,
    name: String,
}

impl Buffer {
    pub fn new(
        memory: &MemoryContext,
        name: &str,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        sharing: &QueueSharing,
    ) -> Result<Buffer, MemoryError> {
        let create_info = Buffer::create_info(size, usage, sharing);

        Buffer::from_create_info(memory, name, &create_info, location)
    }

    /// A buffer whose creation also reads `extension`, chained onto its create info: a video profile list, for
    /// example.
    pub fn new_extended<T: vk::ExtendsBufferCreateInfo + ?Sized>(
        memory: &MemoryContext,
        name: &str,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        sharing: &QueueSharing,
        extension: &mut T,
    ) -> Result<Buffer, MemoryError> {
        let create_info = Buffer::create_info(size, usage, sharing).push_next(extension);

        Buffer::from_create_info(memory, name, &create_info, location)
    }

    fn create_info(size: u64, usage: vk::BufferUsageFlags, sharing: &QueueSharing) -> vk::BufferCreateInfo<'_> {
        vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(sharing.mode())
            .queue_family_indices(sharing.families())
    }

    fn from_create_info(
        memory: &MemoryContext,
        name: &str,
        create_info: &vk::BufferCreateInfo,
        location: MemoryLocation,
    ) -> Result<Buffer, MemoryError> {
        let device = memory.device();
        let size = create_info.size;
        let usage = create_info.usage;
        let buffer = DeviceObject::new(memory, unsafe { device.create_buffer(create_info, None) }?);
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer.handle()) };
        let allocation = memory.allocate(
            name,
            requirements,
            location,
            true,
            AllocationScheme::GpuAllocatorManaged,
        )?;
        unsafe { device.bind_buffer_memory(buffer.handle(), allocation.get().memory(), allocation.get().offset()) }?;
        let device_address = usage
            .contains(vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .then(|| unsafe {
                device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(buffer.handle()))
            });

        Ok(Buffer {
            buffer,
            allocation,
            device_address,
            size,
            name: name.to_owned(),
        })
    }

    /// A buffer the CPU writes and the GPU reads, holding `data`.
    pub fn with_data<T: Copy>(
        memory: &MemoryContext,
        name: &str,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> Result<Buffer, MemoryError> {
        let mut buffer = Buffer::new(
            memory,
            name,
            size_of_val(data) as u64,
            usage,
            MemoryLocation::CpuToGpu,
            &QueueSharing::Exclusive,
        )?;
        buffer.write(data)?;

        Ok(buffer)
    }

    pub fn handle(&self) -> vk::Buffer {
        self.buffer.handle()
    }

    /// The size it was created with, in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// # Panics
    ///
    /// If the buffer was created without `SHADER_DEVICE_ADDRESS` usage.
    pub fn device_address(&self) -> vk::DeviceAddress {
        self.device_address
            .expect("the buffer was created without SHADER_DEVICE_ADDRESS usage")
    }

    pub fn write<T: Copy>(&mut self, data: &[T]) -> Result<(), MemoryError> {
        self.write_at(0, data)
    }

    pub(crate) fn write_at<T: Copy>(&mut self, offset: usize, data: &[T]) -> Result<(), MemoryError> {
        let size = size_of_val(data);
        let destination = self
            .allocation
            .get_mut()
            .mapped_slice_mut()
            .ok_or_else(|| Buffer::host_access(&self.name, "the CPU cannot access it"))?;

        if offset + size > destination.len() {
            return Err(Buffer::host_access(&self.name, "writing past its end"));
        }

        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), destination[offset..].as_mut_ptr(), size) };

        Ok(())
    }

    pub fn read<T: Copy>(&self, data: &mut [T]) -> Result<(), MemoryError> {
        self.read_at(0, data)
    }

    pub(crate) fn read_at<T: Copy>(&self, offset: usize, data: &mut [T]) -> Result<(), MemoryError> {
        let size = size_of_val(data);
        let source = self
            .allocation
            .get()
            .mapped_slice()
            .ok_or_else(|| Buffer::host_access(&self.name, "the CPU cannot access it"))?;

        if offset + size > source.len() {
            return Err(Buffer::host_access(&self.name, "reading past its end"));
        }

        unsafe { std::ptr::copy_nonoverlapping(source[offset..].as_ptr(), data.as_mut_ptr().cast::<u8>(), size) };

        Ok(())
    }

    fn host_access(name: &str, reason: &'static str) -> MemoryError {
        MemoryError::HostAccess {
            buffer: name.to_owned(),
            reason,
        }
    }
}
