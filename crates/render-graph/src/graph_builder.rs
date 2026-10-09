use gpu_allocator::vulkan::Allocator;

use crate::recorded_pass::RecordedPass;
use crate::{
    BufferHandle, CompiledFrame, ImageHandle, ImportedBuffer, ImportedImage, RenderGraph, RenderGraphError, RenderPass,
};

/// Collects one frame's passes, in the order they run.
pub struct GraphBuilder<'graph, 'passes, Context> {
    graph: &'graph mut RenderGraph,
    passes: Vec<&'passes mut dyn RecordedPass<Context>>,
}

impl<'graph, 'passes, Context> GraphBuilder<'graph, 'passes, Context> {
    pub(crate) fn new(graph: &'graph mut RenderGraph) -> GraphBuilder<'graph, 'passes, Context> {
        graph.begin_declaration();

        GraphBuilder {
            graph,
            passes: Vec::new(),
        }
    }

    pub fn import_image(&mut self, image: ImportedImage) -> ImageHandle {
        self.graph.import_image(image)
    }

    pub fn import_buffer(&mut self, buffer: ImportedBuffer) -> BufferHandle {
        self.graph.import_buffer(buffer)
    }

    /// Adds a pass after the ones already added and returns what it gives later passes.
    pub fn add<Pass: RenderPass<Context> + 'passes>(
        &mut self,
        pass: &'passes mut Pass,
        inputs: Pass::Inputs,
    ) -> Pass::Outputs {
        let outputs = self.graph.declare(&*pass, inputs);
        self.passes.push(pass);

        outputs
    }

    /// Compiles the frame, or reuses an earlier frame's result when the declarations match, and allocates the images
    /// it needs.
    pub fn compile(
        self,
        device: &ash::Device,
        allocator: &mut Allocator,
    ) -> Result<CompiledFrame<'graph, 'passes, Context>, RenderGraphError> {
        let mut created = Vec::new();
        let entry = self.graph.compile(device, allocator, &mut created)?;

        Ok(CompiledFrame::new(self.graph, self.passes, entry, created))
    }
}
