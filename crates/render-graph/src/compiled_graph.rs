use ash::vk;

use crate::buffer_end_state::BufferEndState;
use crate::compiled_buffer_barrier::CompiledBufferBarrier;
use crate::compiled_image_barrier::CompiledImageBarrier;
use crate::compiled_pass::CompiledPass;
use crate::compiled_step::CompiledStep;
use crate::frame_declaration::FrameDeclaration;
use crate::image_end_state::ImageEndState;

// The passes that run, in order, with the barriers before each of their steps.
pub(crate) struct CompiledGraph {
    pub passes: Vec<CompiledPass>,
    pub final_image_barriers: Vec<CompiledImageBarrier>,
    pub final_buffer_barriers: Vec<CompiledBufferBarrier>,
    pub image_end_states: Vec<ImageEndState>,
    pub image_usage_flags: Vec<vk::ImageUsageFlags>,
    pub buffer_end_states: Vec<BufferEndState>,
    pub buffer_usage_flags: Vec<vk::BufferUsageFlags>,
}

impl CompiledGraph {
    pub fn new(
        declaration: &FrameDeclaration,
        live_passes: &[usize],
        image_usage_flags: Vec<vk::ImageUsageFlags>,
        buffer_usage_flags: Vec<vk::BufferUsageFlags>,
    ) -> Self {
        let passes = live_passes
            .iter()
            .map(|pass| CompiledPass {
                declaration: *pass,
                steps: vec![CompiledStep::default(); declaration.passes[*pass].step_count as usize],
            })
            .collect();

        CompiledGraph {
            passes,
            final_image_barriers: Vec::new(),
            final_buffer_barriers: Vec::new(),
            image_end_states: Vec::new(),
            image_usage_flags,
            buffer_end_states: Vec::new(),
            buffer_usage_flags,
        }
    }

    pub fn merge_levels(&mut self) {
        for step in self.passes.iter_mut().flat_map(|pass| &mut pass.steps) {
            CompiledImageBarrier::merge_levels(&mut step.image_barriers);
        }

        CompiledImageBarrier::merge_levels(&mut self.final_image_barriers);
    }
}
