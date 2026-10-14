use ash::vk;

use super::GpuContext;
use super::gpu_data::TerrainData;
use crate::vulkan::Buffer;

pub(in crate::renderer) struct TerrainBuffer {
    buffer: Buffer,
}

impl TerrainBuffer {
    pub fn new(gpu: &mut GpuContext, name: &str, data: TerrainData) -> anyhow::Result<Self> {
        let buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
            name,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            &[data],
        )?;

        Ok(Self { buffer })
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.buffer.destroy(&gpu.device, &mut gpu.allocator) };
    }
}
