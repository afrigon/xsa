use ash::vk;

use crate::access::Access;
use crate::{Attachment, Stage};

const DEPTH_TESTS: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::from_raw(
    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS.as_raw() | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS.as_raw(),
);
const TRANSFER_READ_STAGES: vk::PipelineStageFlags2 =
    vk::PipelineStageFlags2::from_raw(vk::PipelineStageFlags2::COPY.as_raw() | vk::PipelineStageFlags2::BLIT.as_raw());
const TRANSFER_WRITE_STAGES: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::from_raw(
    vk::PipelineStageFlags2::COPY.as_raw()
        | vk::PipelineStageFlags2::BLIT.as_raw()
        | vk::PipelineStageFlags2::CLEAR.as_raw(),
);

/// How a step uses an image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourceUsage {
    ColorAttachment(Attachment),
    DepthAttachment(Attachment),
    Sampled(Stage),
    StorageRead(Stage),
    StorageWrite(Stage),
    StorageReadWrite(Stage),
    TransferSource,
    TransferDestination,
}

impl ResourceUsage {
    pub(crate) fn reads(self) -> bool {
        match self {
            ResourceUsage::ColorAttachment(attachment) | ResourceUsage::DepthAttachment(attachment) => {
                attachment.preserves_contents()
            }
            ResourceUsage::Sampled(_)
            | ResourceUsage::StorageRead(_)
            | ResourceUsage::StorageReadWrite(_)
            | ResourceUsage::TransferSource => true,
            ResourceUsage::StorageWrite(_) | ResourceUsage::TransferDestination => false,
        }
    }

    pub(crate) fn writes(self) -> bool {
        match self {
            ResourceUsage::ColorAttachment(_)
            | ResourceUsage::DepthAttachment(_)
            | ResourceUsage::StorageWrite(_)
            | ResourceUsage::StorageReadWrite(_)
            | ResourceUsage::TransferDestination => true,
            ResourceUsage::Sampled(_) | ResourceUsage::StorageRead(_) | ResourceUsage::TransferSource => false,
        }
    }

    pub(crate) fn attachment(self) -> Option<Attachment> {
        match self {
            ResourceUsage::ColorAttachment(attachment) | ResourceUsage::DepthAttachment(attachment) => Some(attachment),
            _ => None,
        }
    }

    pub(crate) fn image_usage(self) -> vk::ImageUsageFlags {
        match self {
            ResourceUsage::ColorAttachment(_) => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ResourceUsage::DepthAttachment(_) => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            ResourceUsage::Sampled(_) => vk::ImageUsageFlags::SAMPLED,
            ResourceUsage::StorageRead(_) | ResourceUsage::StorageWrite(_) | ResourceUsage::StorageReadWrite(_) => {
                vk::ImageUsageFlags::STORAGE
            }
            ResourceUsage::TransferSource => vk::ImageUsageFlags::TRANSFER_SRC,
            ResourceUsage::TransferDestination => vk::ImageUsageFlags::TRANSFER_DST,
        }
    }

    pub(crate) fn access(self, sampled_layout: vk::ImageLayout) -> Access {
        match self {
            ResourceUsage::ColorAttachment(_) => Access {
                stages: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                access: vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            ResourceUsage::DepthAttachment(_) => Access {
                stages: DEPTH_TESTS,
                access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
            ResourceUsage::Sampled(stage) => Access {
                stages: stage.pipeline_stages(),
                access: vk::AccessFlags2::SHADER_SAMPLED_READ,
                layout: sampled_layout,
            },
            ResourceUsage::StorageRead(stage) => Access {
                stages: stage.pipeline_stages(),
                access: vk::AccessFlags2::SHADER_STORAGE_READ,
                layout: vk::ImageLayout::GENERAL,
            },
            ResourceUsage::StorageWrite(stage) => Access {
                stages: stage.pipeline_stages(),
                access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
                layout: vk::ImageLayout::GENERAL,
            },
            ResourceUsage::StorageReadWrite(stage) => Access {
                stages: stage.pipeline_stages(),
                access: vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                layout: vk::ImageLayout::GENERAL,
            },
            ResourceUsage::TransferSource => Access {
                stages: TRANSFER_READ_STAGES,
                access: vk::AccessFlags2::TRANSFER_READ,
                layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            },
            ResourceUsage::TransferDestination => Access {
                stages: TRANSFER_WRITE_STAGES,
                access: vk::AccessFlags2::TRANSFER_WRITE,
                layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            },
        }
    }
}
