use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};

use crate::{BufferState, GraphBufferDescription, RenderGraphError};

const BASE_USAGE: vk::BufferUsageFlags = vk::BufferUsageFlags::from_raw(
    vk::BufferUsageFlags::STORAGE_BUFFER.as_raw() | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS.as_raw(),
);

pub(crate) struct PhysicalBuffer {
    pub buffer: vk::Buffer,
    pub address: vk::DeviceAddress,
    pub description: GraphBufferDescription,
    pub state: BufferState,
    allocation: Option<Allocation>,
}

impl PhysicalBuffer {
    pub fn new(
        device: &ash::Device,
        allocator: &mut Allocator,
        description: GraphBufferDescription,
        usage: vk::BufferUsageFlags,
    ) -> Result<PhysicalBuffer, RenderGraphError> {
        let create_info = vk::BufferCreateInfo::default()
            .size(description.size)
            .usage(usage | BASE_USAGE)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&create_info, None) }?;
        let mut physical = PhysicalBuffer {
            buffer,
            address: 0,
            description,
            state: BufferState::UNUSED,
            allocation: None,
        };

        if let Err(creation) = physical.bind_memory(device, allocator) {
            return match unsafe { physical.destroy(device, allocator) } {
                Ok(()) => Err(creation),
                Err(cleanup) => Err(RenderGraphError::BufferCleanup {
                    buffer: description.name,
                    creation: Box::new(creation),
                    cleanup: Box::new(cleanup),
                }),
            };
        }

        Ok(physical)
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        unsafe { device.destroy_buffer(self.buffer, None) };

        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation)?;
        }

        Ok(())
    }

    fn bind_memory(&mut self, device: &ash::Device, allocator: &mut Allocator) -> Result<(), RenderGraphError> {
        let requirements = unsafe { device.get_buffer_memory_requirements(self.buffer) };
        let allocation = allocator.allocate(&AllocationCreateDesc {
            name: self.description.name,
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        let bound = unsafe { device.bind_buffer_memory(self.buffer, allocation.memory(), allocation.offset()) };
        self.allocation = Some(allocation);
        bound?;
        self.address =
            unsafe { device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(self.buffer)) };

        Ok(())
    }
}
