use ash::vk;

use crate::recorded_pass::RecordedPass;
use crate::{ImageId, RenderGraph, RenderGraphError};

/// A frame ready to record. The images it uses exist, so their ids can be written into the host's frame data first.
pub struct CompiledFrame<'graph, 'passes, Context> {
    graph: &'graph mut RenderGraph,
    passes: Vec<Box<dyn RecordedPass<Context> + 'passes>>,
    entry: usize,
    created: Vec<ImageId>,
}

impl<'graph, 'passes, Context> CompiledFrame<'graph, 'passes, Context> {
    pub(crate) fn new(
        graph: &'graph mut RenderGraph,
        passes: Vec<Box<dyn RecordedPass<Context> + 'passes>>,
        entry: usize,
        created: Vec<ImageId>,
    ) -> CompiledFrame<'graph, 'passes, Context> {
        CompiledFrame {
            graph,
            passes,
            entry,
            created,
        }
    }

    /// Images allocated or replaced by this compile, whose views the host has not seen yet.
    pub fn created_images(&self) -> &[ImageId] {
        &self.created
    }

    pub fn graph(&self) -> &RenderGraph {
        self.graph
    }

    /// Records every pass that runs, with the barriers before each step and the final transitions of imports.
    ///
    /// A frame whose execution fails must not be submitted: the graph keeps the state its images and buffers ended
    /// the previous frame in, and the next frame's barriers start from it.
    pub fn execute(
        mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        context: &Context,
    ) -> Result<(), RenderGraphError> {
        self.graph
            .execute(self.entry, &mut self.passes, device, command_buffer, context)
    }
}
