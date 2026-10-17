use ash::vk;
use render_graph_vulkan::{GraphBuilder, RenderGraph};

use super::ShaderBinaries;
use super::frame_context::FrameContext;
use super::frame_inputs::FrameInputs;
use super::gpu_context::GpuContext;
use super::passes::{
    BloomPass, CaptureInputs, CapturePass, ForwardPass, HistogramInputs, HistogramPass, LensInputs, LensPass,
    SkyboxPass, TaaPass, TonemapInputs, TonemapPass, UserInterfacePass,
};
use crate::config::{AntialiasingKind, RenderConfig};

// Every pass of the renderer, and the order a frame runs them in.
pub(super) struct RenderPasses {
    forward: ForwardPass,
    histogram: HistogramPass,
    skybox: SkyboxPass,
    taa: TaaPass,
    bloom: BloomPass,
    lens: LensPass,
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
            histogram: HistogramPass::new(gpu, shaders)?,
            skybox: SkyboxPass::new(gpu, shaders)?,
            taa: TaaPass::new(gpu, graph, shaders)?,
            bloom: BloomPass::new(gpu, shaders)?,
            lens: LensPass::new(gpu, shaders)?,
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
        let light_count = inputs.projected_light_count;
        let lens = (render.lens.enabled && inputs.lens_loaded && light_count > 0)
            .then(|| graph.add(&mut self.lens, LensInputs { light_count }).lens);
        let forward = graph.add(&mut self.forward, ());
        graph.add(
            &mut self.histogram,
            HistogramInputs {
                scene_color: forward.hdr,
                lens,
                histogram: inputs.histogram,
            },
        );
        let sky = graph.add(&mut self.skybox, forward);
        let scene_color = match render.antialiasing.kind {
            Some(AntialiasingKind::Taa) => graph.add(&mut self.taa, sky).history.current,
            None => sky.hdr,
        };
        let bloom = render
            .bloom
            .enabled
            .then(|| graph.add(&mut self.bloom, scene_color).bloom);
        graph.add(
            &mut self.tonemap,
            TonemapInputs {
                scene_color,
                bloom,
                lens,
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
            self.histogram.destroy(gpu);
            self.skybox.destroy(gpu);
            self.taa.destroy(gpu);
            self.bloom.destroy(gpu);
            self.lens.destroy(gpu);
            self.tonemap.destroy(gpu);
            self.user_interface.destroy();
        }
    }
}
