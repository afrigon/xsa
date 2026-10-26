mod scatter_inputs;

pub(in crate::renderer) use scatter_inputs::ScatterInputs;

use render_graph_vulkan::{BufferUsage, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::{ScatterDrawPushConstants, ScatterPushConstants};
use crate::vulkan::ComputePipeline;

// Must match `workgroupSize` in scatter/draws.slang.
const DRAWS_WORKGROUP_SIZE: u32 = 64;

// Culls the frame's scatter pages into an instance list per model: one workgroup per page places, fades and culls its
// instances; then each submesh's indirect arguments take its model's count, and the counters are copied back.
pub(in crate::renderer) struct ScatterPass {
    cull: ComputePipeline,
    draws: ComputePipeline,
}

impl ScatterPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<ScatterPass> {
        let layouts = [gpu.bindless.layout()];

        Ok(ScatterPass {
            cull: ComputePipeline::new(
                &gpu.memory,
                &binaries.scatter_cull,
                size_of::<ScatterPushConstants>() as u32,
                &layouts,
            )?,
            draws: ComputePipeline::new(
                &gpu.memory,
                &binaries.scatter_draws,
                size_of::<ScatterDrawPushConstants>() as u32,
                &layouts,
            )?,
        })
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for ScatterPass {
    const NAME: &'static str = "scatter";

    type Inputs = ScatterInputs;
    type Resources = ScatterInputs;

    fn declare(&self, pass: &mut PassDeclaration, inputs: ScatterInputs) -> ScatterInputs {
        pass.buffer(inputs.counts, BufferUsage::TransferDestination);
        pass.next_step();
        pass.buffer(inputs.counts, BufferUsage::StorageReadWrite(Stage::Compute));
        pass.buffer(inputs.instances, BufferUsage::StorageWrite(Stage::Compute));
        pass.next_step();
        pass.buffer(inputs.counts, BufferUsage::StorageRead(Stage::Compute));
        pass.buffer(inputs.arguments, BufferUsage::StorageWrite(Stage::Compute));
        pass.next_step();
        pass.buffer(inputs.counts, BufferUsage::TransferSource);
        pass.buffer(inputs.readback, BufferUsage::TransferDestination);

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        inputs: &ScatterInputs,
    ) -> Result<(), PassError> {
        let recorder = &frame.recorder;
        let buffers = &frame.frame.scatter;
        let scatter = frame.scatter;
        recorder.clear_buffer(pass.buffer(inputs.counts));
        pass.next_step();
        recorder.bind_compute(&self.cull, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.cull,
            &ScatterPushConstants {
                frame: frame.frame.frame_data.device_address(),
                objects: frame.frame.objects.device_address(),
                terrains: frame.frame.terrains.device_address(),
                pages: buffers.pages.device_address(),
                uses: buffers.uses.device_address(),
                models: buffers.models.device_address(),
                counts: buffers.counts.device_address(),
                instances: buffers.instances.device_address(),
                list_count: scatter.lists.len() as u32,
                padding: 0,
            },
        );
        recorder.dispatch_groups(scatter.pages.len() as u32);
        pass.next_step();

        if !scatter.draws.is_empty() {
            recorder.bind_compute(&self.draws, frame.descriptor_set);
            recorder.push_compute_constants(
                &self.draws,
                &ScatterDrawPushConstants {
                    draws: buffers.draws.device_address(),
                    counts: buffers.counts.device_address(),
                    arguments: buffers.arguments.device_address(),
                    draw_count: scatter.draws.len() as u32,
                    padding: 0,
                },
            );
            recorder.dispatch_groups((scatter.draws.len() as u32).div_ceil(DRAWS_WORKGROUP_SIZE));
        }

        pass.next_step();
        recorder.copy_buffer(
            pass.buffer(inputs.counts),
            pass.buffer(inputs.readback),
            buffers.counts_size(),
        );

        Ok(())
    }
}
