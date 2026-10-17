use ash::vk;
use memory_vulkan::{Buffer, MemoryContext, MemoryLocation, QueueSharing};

use crate::{BufferState, GraphBufferDescription, RenderGraphError};

const BASE_USAGE: vk::BufferUsageFlags = vk::BufferUsageFlags::from_raw(
    vk::BufferUsageFlags::STORAGE_BUFFER.as_raw() | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS.as_raw(),
);

pub(crate) struct PhysicalBuffer {
    buffer: Buffer,
    pub description: GraphBufferDescription,
    pub state: BufferState,
}

impl PhysicalBuffer {
    pub fn new(
        memory: &MemoryContext,
        description: GraphBufferDescription,
        usage: vk::BufferUsageFlags,
    ) -> Result<PhysicalBuffer, RenderGraphError> {
        let buffer = Buffer::new(
            memory,
            description.name,
            description.size,
            usage | BASE_USAGE,
            MemoryLocation::GpuOnly,
            &QueueSharing::Exclusive,
        )?;

        Ok(PhysicalBuffer {
            buffer,
            description,
            state: BufferState::UNUSED,
        })
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer.handle()
    }

    pub fn address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }
}
