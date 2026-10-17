use ash::vk;

use super::GpuContext;
use super::gpu_data::TerrainData;
use super::static_buffer::StaticBuffer;

pub(in crate::renderer) struct TerrainBuffer {
    buffer: StaticBuffer,
    uploaded: bool,
}

impl TerrainBuffer {
    pub fn new(gpu: &GpuContext, name: &str, data: TerrainData) -> anyhow::Result<Self> {
        let buffer = StaticBuffer::new(
            gpu,
            name,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            &[data],
        )?;

        Ok(Self {
            buffer,
            uploaded: false,
        })
    }

    // None until its upload completes.
    pub fn device_address(&self) -> Option<vk::DeviceAddress> {
        self.uploaded.then(|| self.buffer.device_address())
    }

    pub fn poll(&mut self) -> anyhow::Result<()> {
        self.uploaded = self.buffer.poll()?;
        Ok(())
    }
}
