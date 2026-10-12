use ash::vk;

use super::{Material, Shader};
use crate::renderer::GpuContext;
use crate::vulkan::Buffer;

pub(in crate::renderer) struct MaterialBuffer {
    shader: Shader,
    buffer: Buffer,
}

impl MaterialBuffer {
    pub fn new<M: Material>(gpu: &mut GpuContext, name: &str, material: &M) -> anyhow::Result<Self> {
        let buffer = Buffer::with_data(
            &gpu.device,
            &mut gpu.allocator,
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

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.buffer.destroy(&gpu.device, &mut gpu.allocator) };
    }
}
