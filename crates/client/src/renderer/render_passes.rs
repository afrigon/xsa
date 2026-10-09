use ash::vk;
use render_graph::{GraphBuilder, RenderGraph};

use super::ShaderBinaries;
use super::frame_context::FrameContext;
use super::frame_inputs::FrameInputs;
use super::gpu_context::GpuContext;
use super::passes::{
    BloomPass, CaptureInputs, CapturePass, ForwardPass, HistogramInputs, HistogramPass, TaaPass, TonemapInputs,
    TonemapPass, UserInterfacePass,
};
use crate::config::{AntialiasingKind, RenderConfig};

// Every pass of the renderer, and the order a frame runs them in.
pub(super) struct RenderPasses {
    forward: ForwardPass,
    taa: TaaPass,
    histogram: HistogramPass,
    bloom: BloomPass,
    tonemap: TonemapPass,
    user_interface: UserInterfacePass,
    capture: CapturePass,
}

impl RenderPasses {
    pub fn new(
        gpu: &mut GpuContext,
        graph: &mut RenderGraph,
        shaders: &ShaderBinaries,
        output_format: vk::Format,
        frames_in_flight: usize,
    ) -> anyhow::Result<RenderPasses> {
        Ok(RenderPasses {
            forward: ForwardPass::new(gpu, shaders)?,
            taa: TaaPass::new(gpu, graph, shaders)?,
            histogram: HistogramPass::new(gpu, shaders)?,
            bloom: BloomPass::new(gpu, shaders)?,
            tonemap: TonemapPass::new(gpu, shaders, output_format)?,
            user_interface: UserInterfacePass::new(gpu, output_format, frames_in_flight)?,
            capture: CapturePass,
        })
    }

    pub fn add_to<'passes, 'frame>(
        &'passes mut self,
        graph: &mut GraphBuilder<'_, 'passes, FrameContext<'frame>>,
        render: &RenderConfig,
        inputs: FrameInputs,
    ) {
        let forward = graph.add(&mut self.forward, ());
        let scene_color = match render.antialiasing.kind {
            Some(AntialiasingKind::Taa) => graph.add(&mut self.taa, forward).history.current,
            None => forward.hdr,
        };
        graph.add(
            &mut self.histogram,
            HistogramInputs {
                scene_color,
                histogram: inputs.histogram,
            },
        );
        let bloom = render
            .bloom
            .enabled
            .then(|| graph.add(&mut self.bloom, scene_color).bloom);
        graph.add(
            &mut self.tonemap,
            TonemapInputs {
                scene_color,
                bloom,
                output: inputs.output,
            },
        );
        graph.add(&mut self.user_interface, inputs.output);

        if let Some(capture) = inputs.capture {
            graph.add(
                &mut self.capture,
                CaptureInputs {
                    output: inputs.output,
                    capture,
                },
            );
        }
    }

    // Safety: only once the GPU has finished every frame that used the passes.
    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            self.forward.destroy(gpu);
            self.taa.destroy(gpu);
            self.histogram.destroy(gpu);
            self.bloom.destroy(gpu);
            self.tonemap.destroy(gpu);
            self.user_interface.destroy();
        }
    }
}
