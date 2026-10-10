use ash::vk;

use crate::image_barrier_source::ImageBarrierSource;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompiledImageBarrier {
    pub image: usize,
    pub base_level: u32,
    pub level_count: u32,
    pub source: ImageBarrierSource,
    pub destination_stages: vk::PipelineStageFlags2,
    pub destination_access: vk::AccessFlags2,
    pub new_layout: vk::ImageLayout,
}

impl CompiledImageBarrier {
    pub fn merge_levels(barriers: &mut Vec<CompiledImageBarrier>) {
        let mut merged: Vec<CompiledImageBarrier> = Vec::with_capacity(barriers.len());

        for barrier in barriers.drain(..) {
            if let Some(last) = merged.last_mut()
                && last.extends_to(&barrier)
            {
                last.level_count += barrier.level_count;
                continue;
            }

            merged.push(barrier);
        }

        *barriers = merged;
    }

    // Levels keeping their previous layout may each have been left in a different one.
    fn extends_to(&self, next: &CompiledImageBarrier) -> bool {
        self.image == next.image
            && self.base_level + self.level_count == next.base_level
            && self.source == next.source
            && self.source != ImageBarrierSource::PreviousFrame { discard: false }
            && self.destination_stages == next.destination_stages
            && self.destination_access == next.destination_access
            && self.new_layout == next.new_layout
    }
}
