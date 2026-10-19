use ash::vk;

use super::{Material, MaterialData, Shader};
use crate::renderer::GpuContext;
use crate::renderer::static_buffer::StaticBuffer;

// A material's parameters in GPU memory. They change when a texture they name becomes ready: a new buffer is then
// uploaded and replaces the drawn one, with the shader that reads it, once its copy completes.
pub(in crate::renderer) struct MaterialBuffer {
    name: String,
    material: Box<dyn Material>,
    // The data of the latest upload.
    uploaded: Option<MaterialData>,
    drawn: Option<UploadedMaterial>,
    uploading: Option<UploadedMaterial>,
}

struct UploadedMaterial {
    shader: Shader,
    buffer: StaticBuffer,
}

impl MaterialBuffer {
    pub fn new<M: Material + 'static>(gpu: &GpuContext, name: &str, material: M) -> anyhow::Result<MaterialBuffer> {
        let mut buffer = MaterialBuffer {
            name: name.to_owned(),
            material: Box::new(material),
            uploaded: None,
            drawn: None,
            uploading: None,
        };
        buffer.refresh(gpu)?;

        Ok(buffer)
    }

    // None until its first upload completes.
    pub fn shader(&self) -> Option<Shader> {
        self.drawn.as_ref().map(|drawn| drawn.shader)
    }

    // None until its first upload completes.
    pub fn device_address(&self) -> Option<vk::DeviceAddress> {
        self.drawn.as_ref().map(|drawn| drawn.buffer.device_address())
    }

    pub fn refresh(&mut self, gpu: &GpuContext) -> anyhow::Result<()> {
        if let Some(uploading) = &mut self.uploading
            && uploading.buffer.poll()?
        {
            self.drawn = self.uploading.take();
        }

        let Some(data) = self.material.gpu_data() else {
            return Ok(());
        };

        if self.uploaded.as_ref() != Some(&data) {
            self.uploading = Some(UploadedMaterial {
                shader: data.shader,
                buffer: StaticBuffer::new(
                    gpu,
                    &self.name,
                    vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                    &data.bytes,
                )?,
            });
            self.uploaded = Some(data);
        }

        Ok(())
    }
}
