use std::num::NonZeroU32;
use std::sync::Arc;

use ash::vk;
use memory_vulkan::{
    BindlessBinding, BindlessSlot, BindlessTable, GpuUpload, Image, ImageDescription, ImageLayers, ImageMemory,
};

use crate::LoadEnvironment;
use crate::streaming::{ChunkId, PageFormat};

// One texture array of a pool, a page per layer, read by shaders through its bindless slot. It stays in `GENERAL`
// layout, where layers are written while frames sample the others. Its memory is its own, so adding or releasing it
// moves video memory usage by its size.
pub(crate) struct Chunk {
    pub id: ChunkId,
    pub layers: u32,
    // Bytes of video memory the driver allocated for it.
    pub size: u64,
    pub created_in: u64,
    state: ChunkState,
}

// Declaration order is destruction order: the slot returns once frames reading the image finish.
enum ChunkState {
    Preparing {
        slot: BindlessSlot,
        upload: GpuUpload<Image>,
    },
    Ready {
        slot: BindlessSlot,
        image: Arc<Image>,
    },
    Failed,
}

impl Chunk {
    pub fn new(
        environment: &LoadEnvironment,
        format: &PageFormat,
        layers: NonZeroU32,
        id: ChunkId,
        now: u64,
    ) -> anyhow::Result<Chunk> {
        let image = Image::new(
            &environment.memory,
            &ImageDescription {
                name: "streamed pages",
                extent: format.extent,
                format: format.format,
                usage: vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::TRANSFER_SRC,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                layers: ImageLayers::Array { count: layers },
                memory: ImageMemory::Dedicated,
                sharing: environment.transfer.sharing(),
            },
        )?;
        let size = image.size();
        let slot = environment.bindless.allocate(BindlessBinding::TextureArray)?;
        let upload = environment
            .transfer
            .prepare_image(image, vk::ImageLayout::GENERAL, memory_vulkan::TransferPriority::High)
            .into();

        Ok(Chunk {
            id,
            layers: layers.get(),
            size,
            created_in: now,
            state: ChunkState::Preparing { slot, upload },
        })
    }

    // True once it became ready in this call.
    pub fn poll(&mut self, bindless: &BindlessTable) -> bool {
        let ChunkState::Preparing { upload, .. } = &mut self.state else {
            return false;
        };

        match upload.poll() {
            Ok(None) => false,
            Ok(Some(_)) => {
                let ChunkState::Preparing {
                    slot,
                    upload: GpuUpload::Ready(image),
                } = std::mem::replace(&mut self.state, ChunkState::Failed)
                else {
                    unreachable!("a polled ready chunk holds its image");
                };
                bindless.write(&slot, image.view(), vk::ImageLayout::GENERAL);
                self.state = ChunkState::Ready {
                    slot,
                    image: Arc::new(image),
                };
                true
            }
            Err(err) => {
                tracing::warn!("preparing a chunk of streamed pages failed: {err}");
                self.state = ChunkState::Failed;
                false
            }
        }
    }

    pub fn is_preparing(&self) -> bool {
        matches!(self.state, ChunkState::Preparing { .. })
    }

    // Its image and bindless index, once ready.
    pub fn ready(&self) -> Option<(&Arc<Image>, u32)> {
        match &self.state {
            ChunkState::Ready { slot, image } => Some((image, slot.index())),
            ChunkState::Preparing { .. } | ChunkState::Failed => None,
        }
    }
}
