mod taa_resources;

pub(in crate::renderer) use taa_resources::TaaResources;

use ash::vk;
use render_graph::{
    GraphImageDescription, HistoryId, ImageSize, PassContext, PassDeclaration, PassError, RenderGraph, RenderPass,
    Stage,
};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::TaaPushConstants;
use crate::renderer::passes::ForwardResources;
use crate::vulkan::ComputePipeline;

const HISTORY_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const HISTORY_LEVEL: u32 = 0;
// Must match the workgroup size in taa.slang.
const WORKGROUP_SIZE: u32 = 8;
// Each frame adds a tenth to the history: about ten frames of samples, few enough to follow changes quickly.
const CURRENT_FRAME_WEIGHT: f32 = 0.1;
// A history is settled once what it held before carries less than this share of the result.
const SETTLED_HISTORY_SHARE: f32 = 0.01;

// Temporal anti-aliasing: blends each jittered frame into a history reprojected along the motion vectors. The history
// pair alternates between being read as last frame's result and written as this frame's.
pub(in crate::renderer) struct TaaPass {
    pipeline: ComputePipeline,
    history: HistoryId,
}

impl TaaPass {
    pub fn new(gpu: &GpuContext, graph: &mut RenderGraph, binaries: &ShaderBinaries) -> anyhow::Result<TaaPass> {
        let pipeline = ComputePipeline::new(
            &gpu.device,
            &binaries.taa,
            size_of::<TaaPushConstants>() as u32,
            &[gpu.bindless.layout()],
        )?;
        let history = graph.create_history(GraphImageDescription {
            name: "taa history",
            format: HISTORY_FORMAT,
            size: ImageSize::Output,
            mip_levels: 1,
        });

        Ok(TaaPass { pipeline, history })
    }

    // Frames until the history has replaced all but SETTLED_HISTORY_SHARE of what it held before.
    pub fn settling_frames() -> u32 {
        (SETTLED_HISTORY_SHARE.ln() / (1.0 - CURRENT_FRAME_WEIGHT).ln()).ceil() as u32
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for TaaPass {
    const NAME: &'static str = "taa";

    type Inputs = ForwardResources;
    type Resources = TaaResources;

    fn declare(&self, pass: &mut PassDeclaration, forward: ForwardResources) -> TaaResources {
        let history = pass.history(self.history);
        pass.sampled(forward.hdr, Stage::Compute);
        pass.sampled(forward.motion, Stage::Compute);
        pass.sampled(forward.depth, Stage::Compute);
        pass.sampled(history.previous, Stage::Compute);
        pass.storage_write(history.current, Stage::Compute);

        TaaResources { forward, history }
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        resources: &TaaResources,
    ) -> Result<(), PassError> {
        let recorder = &frame.recorder;
        let temporal = frame.temporal;
        let forward = resources.forward;
        let history = resources.history;
        recorder.bind_compute(&self.pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.pipeline,
            &TaaPushConstants {
                frame: frame.frame.frame_data.device_address(),
                color_texture: frame.texture(pass, forward.hdr),
                motion_texture: frame.texture(pass, forward.motion),
                depth_texture: frame.texture(pass, forward.depth),
                history_texture: frame.texture(pass, history.previous),
                output_image: frame.storage_image(pass, history.current, HISTORY_LEVEL),
                history_valid: u32::from(temporal.is_valid()),
                history_exposure_scale: temporal.history_exposure_scale(),
                current_frame_weight: CURRENT_FRAME_WEIGHT,
            },
        );
        recorder.dispatch(pass.extent(history.current), WORKGROUP_SIZE);

        Ok(())
    }
}
