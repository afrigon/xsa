use ash::vk;
use memory_vulkan::{BindlessSlot, BindlessTable, GpuUpload, Image, TransferPriority};

use crate::PendingResource;
use crate::textures::Texture;

/// A texture whose upload is queued. Its slot is written once the upload completes.
pub struct PendingTexture {
    // Both are taken when the texture becomes ready.
    upload: Option<GpuUpload<Image>>,
    slot: Option<BindlessSlot>,
}

impl PendingTexture {
    pub(crate) fn new(upload: GpuUpload<Image>, slot: BindlessSlot) -> PendingTexture {
        PendingTexture {
            upload: Some(upload),
            slot: Some(slot),
        }
    }
}

impl PendingResource<Texture> for PendingTexture {
    /// # Panics
    ///
    /// If polled again after it returned the texture or an error.
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<Texture>> {
        let upload = self
            .upload
            .as_mut()
            .expect("a pending texture is polled until it settles");

        if upload.poll()?.is_none() {
            return Ok(None);
        }

        let (Some(GpuUpload::Ready(image)), Some(slot)) = (self.upload.take(), self.slot.take()) else {
            unreachable!("a polled ready upload holds its image");
        };
        bindless.write(&slot, image.view(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        Ok(Some(Texture::new(image, slot)))
    }

    fn set_priority(&self, priority: TransferPriority) {
        if let Some(upload) = &self.upload {
            upload.set_priority(priority);
        }
    }
}
