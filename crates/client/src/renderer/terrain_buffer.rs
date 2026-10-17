use ash::vk;
use memory_vulkan::Buffer;

use super::GpuContext;
use super::gpu_data::TerrainData;

pub(in crate::renderer) struct TerrainBuffer {
    buffer: Buffer,
}

impl TerrainBuffer {
    pub fn new(gpu: &GpuContext, name: &str, data: TerrainData) -> anyhow::Result<Self> {
        let buffer = Buffer::with_data(
            &gpu.memory,
            name,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            &[data],
        )?;

        Ok(Self { buffer })
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }
}
