use anyhow::ensure;
use ash::vk;

use crate::memory_types::MemoryTypes;

// A buffer the CPU writes directly; coherent memory makes writes visible to the GPU at submission.
pub(crate) struct HostBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped: *mut u8,
    size: usize,
}

impl HostBuffer {
    pub unsafe fn new(
        device: &ash::Device,
        memory_types: &MemoryTypes,
        size: usize,
        usage: vk::BufferUsageFlags,
    ) -> anyhow::Result<HostBuffer> {
        let info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&info, None) }?;
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory = match unsafe {
            memory_types.allocate(
                device,
                requirements,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
        } {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { device.destroy_buffer(buffer, None) };
                return Err(error);
            }
        };
        let mapped = unsafe {
            device.bind_buffer_memory(buffer, memory, 0)?;
            device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())
        };
        let mapped = match mapped {
            Ok(mapped) => mapped.cast::<u8>(),
            Err(error) => {
                unsafe {
                    device.destroy_buffer(buffer, None);
                    device.free_memory(memory, None);
                }
                return Err(error.into());
            }
        };

        Ok(HostBuffer {
            buffer,
            memory,
            mapped,
            size,
        })
    }

    pub fn handle(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn write(&mut self, offset: usize, bytes: &[u8]) -> anyhow::Result<()> {
        ensure!(
            offset + bytes.len() <= self.size,
            "writing {} bytes at {offset} overflows a {}-byte buffer",
            bytes.len(),
            self.size
        );
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.mapped.add(offset), bytes.len()) };

        Ok(())
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.unmap_memory(self.memory);
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}
