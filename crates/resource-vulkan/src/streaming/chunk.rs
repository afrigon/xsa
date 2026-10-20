use std::sync::Arc;

use ash::vk;
use memory_vulkan::{
    BindlessBinding, BindlessSlot, BindlessTable, GpuUpload, Image, ImageDescription, ImageLayers, ImageMemory,
};

use crate::LoadEnvironment;
use crate::streaming::{ChunkId, PageFormat};

// One 2D texture of a pool holding pages side by side in a grid of cells, read by shaders through its bindless slot.
// It stays in `GENERAL` layout, where cells are written while frames sample the others. Its memory is its own, so
// adding or releasing it moves video memory usage by its size.
pub(crate) struct Chunk {
    pub id: ChunkId,
    pub grid: ChunkGrid,
    // Bytes of video memory the driver allocated for it.
    pub size: u64,
    state: ChunkState,
}

// The cells of a chunk, numbered row by row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChunkGrid {
    pub columns: u32,
    pub rows: u32,
    pub page_extent: vk::Extent2D,
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

impl ChunkGrid {
    pub fn cells(&self) -> u32 {
        self.columns * self.rows
    }

    pub fn extent(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.columns * self.page_extent.width,
            height: self.rows * self.page_extent.height,
        }
    }

    // The texel offset of `cell`'s page in the chunk.
    pub fn offset(&self, cell: u32) -> vk::Offset2D {
        vk::Offset2D {
            x: ((cell % self.columns) * self.page_extent.width) as i32,
            y: ((cell / self.columns) * self.page_extent.height) as i32,
        }
    }
}

impl Chunk {
    pub fn new(
        environment: &LoadEnvironment,
        format: &PageFormat,
        grid: ChunkGrid,
        id: ChunkId,
    ) -> anyhow::Result<Chunk> {
        let image = Image::new(
            &environment.memory,
            &ImageDescription {
                name: "streamed pages",
                extent: grid.extent(),
                format: format.format,
                usage: vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::TRANSFER_SRC,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                layers: ImageLayers::Single,
                memory: ImageMemory::Dedicated,
                sharing: environment.transfer.sharing(),
            },
        )?;
        let size = image.size();
        let slot = environment.bindless.allocate(BindlessBinding::Texture)?;
        let upload = environment
            .transfer
            .prepare_image(image, vk::ImageLayout::GENERAL, memory_vulkan::TransferPriority::High)
            .into();

        Ok(Chunk {
            id,
            grid,
            size,
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
