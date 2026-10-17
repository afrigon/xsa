use memory_vulkan::{GpuUpload, Image};

use super::BindlessSlot;
use crate::vulkan::{BindlessTextures, Device, SAMPLED_LAYOUT};

// A texture being uploaded into a reserved bindless slot, whose descriptor is written once the upload completes.
pub(in crate::renderer) struct SlottedTexture {
    slot: BindlessSlot,
    upload: GpuUpload<Image>,
}

impl SlottedTexture {
    pub fn new(slot: BindlessSlot, upload: GpuUpload<Image>) -> SlottedTexture {
        SlottedTexture { slot, upload }
    }

    pub fn slot(&self) -> BindlessSlot {
        self.slot
    }

    // Whether the texture is ready; its descriptor is written when it becomes so.
    pub fn poll(&mut self, bindless: &BindlessTextures, device: &Device) -> anyhow::Result<bool> {
        let ready = self.upload.poll()?.is_some();

        if ready {
            self.bind(bindless, device);
        }

        Ok(ready)
    }

    pub fn wait(&mut self, bindless: &BindlessTextures, device: &Device) -> anyhow::Result<()> {
        self.upload.wait()?;
        self.bind(bindless, device);

        Ok(())
    }

    pub fn into_image(self) -> Option<Image> {
        match self.upload {
            GpuUpload::Pending(_) => None,
            GpuUpload::Ready(image) => Some(image),
        }
    }

    fn bind(&self, bindless: &BindlessTextures, device: &Device) {
        let Some(image) = self.upload.get() else {
            return;
        };

        match self.slot {
            BindlessSlot::Texture(index) => bindless.set_texture(device, index, image.view(), SAMPLED_LAYOUT),
            BindlessSlot::Cube(index) => bindless.set_cube(device, index, image.view()),
        }
    }
}
