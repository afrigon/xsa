use ash::vk;
use render_graph_vulkan::{GraphBuilder, RenderGraph};
use xui::DrawList;

use super::ShaderBinaries;
use super::frame_context::FrameContext;
use super::frame_inputs::FrameInputs;
use super::gpu_context::GpuContext;
use super::passes::{
    BloomPass, CaptureInputs, CapturePass, ForwardInputs, ForwardPass, HistogramInputs, HistogramPass, LensInputs,
    LensPass, ScatterPass, SkyboxPass, TaaPass, TonemapInputs, TonemapPass, UserInterfacePass, VideoConversionPass,
    WindowCopyInputs, WindowCopyPass,
};
use crate::config::{AntialiasingKind, Config};

// Every pass of the renderer, and the order a frame runs them in.
pub(super) struct RenderPasses {
    scatter: ScatterPass,
    forward: ForwardPass,
    histogram: HistogramPass,
    skybox: SkyboxPass,
    taa: TaaPass,
    bloom: BloomPass,
    lens: LensPass,
    tonemap: TonemapPass,
    user_interface: UserInterfacePass,
    capture: CapturePass,
    video_conversion: VideoConversionPass,
    window_copy: WindowCopyPass,
}

impl RenderPasses {
    pub fn new(
        gpu: &GpuContext,
        graph: &mut RenderGraph,
        shaders: &ShaderBinaries,
        output_format: vk::Format,
        frames_in_flight: usize,
    ) -> anyhow::Result<RenderPasses> {
        Ok(RenderPasses {
            scatter: ScatterPass::new(gpu, shaders)?,
            forward: ForwardPass::new(gpu, shaders)?,
            histogram: HistogramPass::new(gpu, shaders)?,
            skybox: SkyboxPass::new(gpu, shaders)?,
            taa: TaaPass::new(gpu, graph, shaders)?,
            bloom: BloomPass::new(gpu, shaders)?,
            lens: LensPass::new(gpu, shaders)?,
            tonemap: TonemapPass::new(gpu, shaders, output_format)?,
            user_interface: UserInterfacePass::new(gpu, output_format, frames_in_flight)?,
            capture: CapturePass,
            video_conversion: VideoConversionPass,
            window_copy: WindowCopyPass::new(gpu, shaders, output_format)?,
        })
    }

    // Builds what the next frame draws with under `config`, before its recording starts.
    pub fn ensure_pipelines(&mut self, gpu: &GpuContext, config: &Config) -> anyhow::Result<()> {
        self.forward.ensure_pipelines(gpu, &config.render)
    }

    pub fn set_user_interface(&mut self, draw_list: DrawList) {
        self.user_interface.set_draw_list(draw_list);
    }

    pub fn add_to<'passes, 'frame>(
        &'passes mut self,
        graph: &mut GraphBuilder<'_, 'passes, FrameContext<'frame>>,
        config: &Config,
        inputs: FrameInputs,
    ) {
        let light_count = inputs.projected_light_count;
        let lens = (config.video.lens.enabled && inputs.lens_loaded && light_count > 0)
            .then(|| graph.add(&mut self.lens, LensInputs { light_count }).lens);
        let scatter = inputs.scatter.map(|scatter| graph.add(&mut self.scatter, scatter));
        let forward = graph.add(
            &mut self.forward,
            ForwardInputs {
                clear_color: inputs.clear_color.extend(1.0).to_array(),
                scatter,
            },
        );
        graph.add(
            &mut self.histogram,
            HistogramInputs {
                scene_color: forward.hdr,
                lens,
                histogram: inputs.histogram,
            },
        );
        let sky = graph.add(&mut self.skybox, forward);
        let scene_color = match config.video.antialiasing {
            Some(AntialiasingKind::Taa) => graph.add(&mut self.taa, sky).history.current,
            None => sky.hdr,
        };
        let bloom = config
            .video
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

        if let Some(video) = inputs.video {
            graph.add(&mut self.video_conversion, video.frame);

            if let Some(window) = video.window {
                graph.add(
                    &mut self.window_copy,
                    WindowCopyInputs {
                        source: video.frame,
                        window,
                    },
                );
            }
        }
    }
}
