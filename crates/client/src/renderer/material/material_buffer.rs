use ash::vk;

use super::{Material, Shader};
use crate::renderer::GpuContext;
use crate::renderer::static_buffer::StaticBuffer;

pub(in crate::renderer) struct MaterialBuffer {
    shader: Shader,
    buffer: StaticBuffer,
}

impl MaterialBuffer {
    pub fn new<M: Material>(gpu: &GpuContext, name: &str, material: &M) -> anyhow::Result<Self> {
        let buffer = StaticBuffer::new(
            gpu,
            name,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            &[material.gpu_data()],
        )?;

        Ok(Self {
            shader: M::SHADER,
            buffer,
        })
    }

    pub fn shader(&self) -> Shader {
        self.shader
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.buffer.device_address()
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        self.buffer.wait()
    }
}
