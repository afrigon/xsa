use ash::vk;
use memory_vulkan::{Buffer, GpuUpload, MemoryLocation, TransferPriority};

use super::GpuContext;

// A buffer in GPU memory filled once through the transfer queue. Its handle and address are known at once; shaders
// may use them once the upload completed.
pub(in crate::renderer) struct StaticBuffer {
    device_address: Option<vk::DeviceAddress>,
    upload: GpuUpload<Buffer>,
}

impl StaticBuffer {
    pub fn new<T: Copy>(
        gpu: &GpuContext,
        name: &str,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> anyhow::Result<StaticBuffer> {
        let size = size_of_val(data);
        let buffer = Buffer::new(
            &gpu.memory,
            name,
            size as u64,
            usage | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            &gpu.transfer.sharing(),
        )?;
        let device_address = usage
            .contains(vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .then(|| buffer.device_address());
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), size) }.to_vec();
        let upload = gpu
            .transfer
            .upload_buffer(buffer, bytes, TransferPriority::Critical)
            .into();

        Ok(StaticBuffer { device_address, upload })
    }

    // Panics if the buffer was created without `SHADER_DEVICE_ADDRESS` usage.
    pub fn device_address(&self) -> vk::DeviceAddress {
        self.device_address
            .expect("the buffer was created without SHADER_DEVICE_ADDRESS usage")
    }

    pub fn poll(&mut self) -> anyhow::Result<bool> {
        Ok(self.upload.poll()?.is_some())
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        self.upload.wait()?;
        Ok(())
    }

    pub fn into_buffer(self) -> anyhow::Result<Buffer> {
        match self.upload {
            GpuUpload::Pending(ticket) => Ok(ticket.wait()?),
            GpuUpload::Ready(buffer) => Ok(buffer),
        }
    }
}
