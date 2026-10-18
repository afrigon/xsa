use ash::vk;

use crate::TimelineValue;
use crate::transfer_queue::{
    StagedReadback, TransferCommand, TransferCompletion, TransferRequest, TransferWork, UploadDestination, UploadTarget,
};

// The copies of the next batch, gathered before recording: image layout changes before them, copies, layout changes
// after them, and the timeline points readbacks wait for.
#[derive(Default)]
pub(crate) struct BatchBuilder {
    pub bytes: u64,
    before: Vec<vk::ImageMemoryBarrier2<'static>>,
    copies: Vec<TransferCommand>,
    after: Vec<vk::ImageMemoryBarrier2<'static>>,
    waits: Vec<(vk::Semaphore, u64)>,
    pub readbacks: Vec<StagedReadback>,
    pub completions: Vec<Box<dyn TransferCompletion>>,
}

impl BatchBuilder {
    pub fn is_empty(&self) -> bool {
        self.copies.is_empty() && self.completions.is_empty()
    }

    // An owned image leaves its undefined layout before its first copy.
    pub fn begin(&mut self, request: &TransferRequest) {
        if let Some((image, range, _)) = BatchBuilder::owned_image(request) {
            self.before.push(
                vk::ImageMemoryBarrier2::default()
                    .dst_stage_mask(vk::PipelineStageFlags2::COPY)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(image)
                    .subresource_range(range),
            );
        }
    }

    pub fn copy(&mut self, command: TransferCommand, size: usize) {
        self.copies.push(command);
        self.bytes += size as u64;
    }

    pub fn wait_for(&mut self, point: &TimelineValue) {
        let semaphore = point.timeline.semaphore();

        match self.waits.iter_mut().find(|(waited, _)| *waited == semaphore) {
            Some((_, value)) => *value = (*value).max(point.value),
            None => self.waits.push((semaphore, point.value)),
        }
    }

    // The request's last copy is in this batch: an owned image ends in its target layout. The semaphore the batch
    // signals makes every copy visible to the submissions waiting on it, regions included.
    pub fn finish(&mut self, request: TransferRequest) {
        if let Some((image, range, layout)) = BatchBuilder::owned_image(&request) {
            let old_layout = if request.started {
                vk::ImageLayout::TRANSFER_DST_OPTIMAL
            } else {
                vk::ImageLayout::UNDEFINED
            };
            self.after.push(
                vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::COPY)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                    .old_layout(old_layout)
                    .new_layout(layout)
                    .image(image)
                    .subresource_range(range),
            );
        }

        self.completions.push(request.completion);
    }

    pub fn semaphore_waits(&self) -> Vec<vk::SemaphoreSubmitInfo<'static>> {
        self.waits
            .iter()
            .map(|&(semaphore, value)| {
                vk::SemaphoreSubmitInfo::default()
                    .semaphore(semaphore)
                    .value(value)
                    .stage_mask(vk::PipelineStageFlags2::COPY)
            })
            .collect()
    }

    pub unsafe fn record(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            if !self.before.is_empty() {
                device.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(&self.before),
                );
            }

            for copy in &self.copies {
                copy.record(device, command_buffer);
            }

            if !self.after.is_empty() {
                device.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(&self.after),
                );
            }
        }
    }

    fn owned_image(request: &TransferRequest) -> Option<(vk::Image, vk::ImageSubresourceRange, vk::ImageLayout)> {
        match &request.work {
            TransferWork::Upload {
                destination: UploadDestination::Owned(UploadTarget::Image { image, range, layout }),
                ..
            } => Some((*image, *range, *layout)),
            _ => None,
        }
    }
}
