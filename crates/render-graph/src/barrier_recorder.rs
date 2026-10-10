use std::cell::RefCell;

use ash::vk;

use crate::buffer_barrier_source::BufferBarrierSource;
use crate::buffer_pool::BufferPool;
use crate::compiled_buffer_barrier::CompiledBufferBarrier;
use crate::compiled_image_barrier::CompiledImageBarrier;
use crate::image_barrier_source::ImageBarrierSource;
use crate::image_pool::ImagePool;
use crate::resolved_buffer::ResolvedBuffer;
use crate::resolved_image::ResolvedImage;
use crate::{BufferState, ImageState};

pub(crate) struct BarrierRecorder<'frame> {
    pub device: &'frame ash::Device,
    pub command_buffer: vk::CommandBuffer,
    pub resolved_images: &'frame [ResolvedImage],
    pub image_pool: &'frame ImagePool,
    pub resolved_buffers: &'frame [ResolvedBuffer],
    pub buffer_pool: &'frame BufferPool,
    pub image_barriers: &'frame RefCell<Vec<vk::ImageMemoryBarrier2<'static>>>,
    pub buffer_barriers: &'frame RefCell<Vec<vk::BufferMemoryBarrier2<'static>>>,
}

impl BarrierRecorder<'_> {
    pub fn record(&self, images: &[CompiledImageBarrier], buffers: &[CompiledBufferBarrier]) {
        let mut image_barriers = self.image_barriers.borrow_mut();
        let mut buffer_barriers = self.buffer_barriers.borrow_mut();
        image_barriers.clear();
        buffer_barriers.clear();

        for barrier in images {
            let resolved = &self.resolved_images[barrier.image];
            let source = match barrier.source {
                ImageBarrierSource::Known(state) => state,
                ImageBarrierSource::PreviousFrame { discard } => self.previous_state(resolved, barrier, discard),
            };

            if source.stages == vk::PipelineStageFlags2::NONE && source.layout == barrier.new_layout {
                continue;
            }

            image_barriers.push(
                vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(source.stages)
                    .src_access_mask(source.access)
                    .dst_stage_mask(barrier.destination_stages)
                    .dst_access_mask(barrier.destination_access)
                    .old_layout(source.layout)
                    .new_layout(barrier.new_layout)
                    .image(resolved.image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(resolved.aspect)
                            .base_mip_level(barrier.base_level)
                            .level_count(barrier.level_count)
                            .layer_count(vk::REMAINING_ARRAY_LAYERS),
                    ),
            );
        }

        for barrier in buffers {
            let resolved = &self.resolved_buffers[barrier.buffer];
            let source = match barrier.source {
                BufferBarrierSource::Known(state) => state,
                BufferBarrierSource::PreviousFrame => self.previous_buffer_state(resolved),
            };

            if source.stages == vk::PipelineStageFlags2::NONE {
                continue;
            }

            buffer_barriers.push(
                vk::BufferMemoryBarrier2::default()
                    .src_stage_mask(source.stages)
                    .src_access_mask(source.access)
                    .dst_stage_mask(barrier.destination_stages)
                    .dst_access_mask(barrier.destination_access)
                    .buffer(resolved.buffer)
                    .size(vk::WHOLE_SIZE),
            );
        }

        if image_barriers.is_empty() && buffer_barriers.is_empty() {
            return;
        }

        let dependency = vk::DependencyInfo::default()
            .image_memory_barriers(&image_barriers)
            .buffer_memory_barriers(&buffer_barriers);
        unsafe { self.device.cmd_pipeline_barrier2(self.command_buffer, &dependency) };
    }

    // Discarding barriers may span levels left in different states; the others always cover one level.
    fn previous_state(&self, resolved: &ResolvedImage, barrier: &CompiledImageBarrier, discard: bool) -> ImageState {
        let Some(id) = resolved.id else {
            return ImageState::UNUSED;
        };
        let levels = barrier.base_level as usize..(barrier.base_level + barrier.level_count) as usize;
        let previous = self.image_pool.image(id).states[levels]
            .iter()
            .fold(ImageState::UNUSED, |union, state| ImageState {
                stages: union.stages | state.stages,
                access: union.access | state.access,
                layout: state.layout,
            });

        if discard {
            ImageState {
                layout: vk::ImageLayout::UNDEFINED,
                ..previous
            }
        } else {
            previous
        }
    }

    fn previous_buffer_state(&self, resolved: &ResolvedBuffer) -> BufferState {
        resolved
            .id
            .map_or(BufferState::UNUSED, |id| self.buffer_pool.buffer(id).state)
    }
}
