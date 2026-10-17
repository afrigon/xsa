use ash::vk;

use super::GpuContext;
use super::gpu_data::TerrainData;
use super::static_buffer::StaticBuffer;

pub(in crate::renderer) struct TerrainBuffer {
    buffer: StaticBuffer,
}

impl TerrainBuffer {
    pub fn new(gpu: &GpuContext, name: &str, data: TerrainData) -> anyhow::Result<Self> {
        let buffer = StaticBuffer::new(
            gpu,
            name,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            &[data],
        )?;

        Ok(Self { buffer })
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        self.buffer.wait()
    }
}
