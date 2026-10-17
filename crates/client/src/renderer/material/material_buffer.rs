use ash::vk;

use super::{Material, MaterialSource, Shader};
use crate::renderer::GpuContext;
use crate::renderer::static_buffer::StaticBuffer;

// A material's parameters in GPU memory. They change when a texture they name becomes ready: a new buffer is then
// uploaded and replaces the drawn one once its copy completes.
pub(in crate::renderer) struct MaterialBuffer {
    name: String,
    source: Box<dyn MaterialSource>,
    // The bytes of the latest upload.
    uploaded: Vec<u8>,
    drawn: Option<StaticBuffer>,
    uploading: Option<StaticBuffer>,
}

impl MaterialBuffer {
    pub fn new<M: Material + 'static>(gpu: &GpuContext, name: &str, material: M) -> anyhow::Result<MaterialBuffer> {
        let mut buffer = MaterialBuffer {
            name: name.to_owned(),
            source: Box::new(material),
            uploaded: Vec::new(),
            drawn: None,
            uploading: None,
        };
        buffer.refresh(gpu)?;

        Ok(buffer)
    }

    pub fn shader(&self) -> Shader {
        self.source.shader()
    }

    // None until its first upload completes.
    pub fn device_address(&self) -> Option<vk::DeviceAddress> {
        self.drawn.as_ref().map(StaticBuffer::device_address)
    }

    pub fn refresh(&mut self, gpu: &GpuContext) -> anyhow::Result<()> {
        if let Some(uploading) = &mut self.uploading
            && uploading.poll()?
        {
            self.drawn = self.uploading.take();
        }

        let Some(bytes) = self.source.gpu_bytes() else {
            return Ok(());
        };

        if bytes != self.uploaded {
            self.uploading = Some(StaticBuffer::new(
                gpu,
                &self.name,
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                &bytes,
            )?);
            self.uploaded = bytes;
        }

        Ok(())
    }
}
