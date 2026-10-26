use ash::vk;

use crate::Stage;
use crate::access::Access;

const TRANSFER_WRITE_STAGES: vk::PipelineStageFlags2 =
    vk::PipelineStageFlags2::from_raw(vk::PipelineStageFlags2::COPY.as_raw() | vk::PipelineStageFlags2::CLEAR.as_raw());

/// How a step uses a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUsage {
    StorageRead(Stage),
    StorageWrite(Stage),
    StorageReadWrite(Stage),
    TransferSource,
    TransferDestination,
    /// Read as the arguments of indirect draws or dispatches.
    IndirectRead,
}

impl BufferUsage {
    pub(crate) fn reads(self) -> bool {
        match self {
            BufferUsage::StorageRead(_)
            | BufferUsage::StorageReadWrite(_)
            | BufferUsage::TransferSource
            | BufferUsage::IndirectRead => true,
            BufferUsage::StorageWrite(_) | BufferUsage::TransferDestination => false,
        }
    }

    pub(crate) fn writes(self) -> bool {
        match self {
            BufferUsage::StorageWrite(_) | BufferUsage::StorageReadWrite(_) | BufferUsage::TransferDestination => true,
            BufferUsage::StorageRead(_) | BufferUsage::TransferSource | BufferUsage::IndirectRead => false,
        }
    }

    pub(crate) fn buffer_usage(self) -> vk::BufferUsageFlags {
        match self {
            BufferUsage::StorageRead(_) | BufferUsage::StorageWrite(_) | BufferUsage::StorageReadWrite(_) => {
                vk::BufferUsageFlags::STORAGE_BUFFER
            }
            BufferUsage::TransferSource => vk::BufferUsageFlags::TRANSFER_SRC,
            BufferUsage::TransferDestination => vk::BufferUsageFlags::TRANSFER_DST,
            BufferUsage::IndirectRead => vk::BufferUsageFlags::INDIRECT_BUFFER,
        }
    }

    pub(crate) fn access(self) -> Access {
        let (stages, access) = match self {
            BufferUsage::StorageRead(stage) => (stage.pipeline_stages(), vk::AccessFlags2::SHADER_STORAGE_READ),
            BufferUsage::StorageWrite(stage) => (stage.pipeline_stages(), vk::AccessFlags2::SHADER_STORAGE_WRITE),
            BufferUsage::StorageReadWrite(stage) => (
                stage.pipeline_stages(),
                vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            ),
            BufferUsage::TransferSource => (vk::PipelineStageFlags2::COPY, vk::AccessFlags2::TRANSFER_READ),
            BufferUsage::TransferDestination => (TRANSFER_WRITE_STAGES, vk::AccessFlags2::TRANSFER_WRITE),
            BufferUsage::IndirectRead => (
                vk::PipelineStageFlags2::DRAW_INDIRECT,
                vk::AccessFlags2::INDIRECT_COMMAND_READ,
            ),
        };

        Access {
            stages,
            access,
            layout: vk::ImageLayout::UNDEFINED,
        }
    }
}
