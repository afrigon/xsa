use ash::vk;
use memory_vulkan::{BindlessSlot, BindlessTable, GpuUpload, Image};

// A texture being uploaded into a reserved bindless slot, whose descriptor is written once the upload completes.
pub(in crate::renderer) struct SlottedTexture {
    slot: BindlessSlot,
    upload: GpuUpload<Image>,
}

impl SlottedTexture {
    pub fn new(slot: BindlessSlot, upload: GpuUpload<Image>) -> SlottedTexture {
        SlottedTexture { slot, upload }
    }

    pub fn slot(&self) -> &BindlessSlot {
        &self.slot
    }

    // Whether the texture is ready; its descriptor is written when it becomes so.
    pub fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<bool> {
        let ready = self.upload.poll()?.is_some();

        if ready {
            self.bind(bindless);
        }

        Ok(ready)
    }

    pub fn wait(&mut self, bindless: &BindlessTable) -> anyhow::Result<()> {
        self.upload.wait()?;
        self.bind(bindless);

        Ok(())
    }

    pub fn into_parts(self) -> Option<(Image, BindlessSlot)> {
        match self.upload {
            GpuUpload::Pending(_) => None,
            GpuUpload::Ready(image) => Some((image, self.slot)),
        }
    }

    fn bind(&self, bindless: &BindlessTable) {
        if let Some(image) = self.upload.get() {
            bindless.write(&self.slot, image.view(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        }
    }
}
